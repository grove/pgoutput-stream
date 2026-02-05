use anyhow::Result;
use tokio_postgres::{Client, NoTls, SimpleQueryMessage};
use std::time::Duration;
use std::collections::VecDeque;

use crate::decoder::{decode_pgoutput_message, Change};

pub struct ReplicationStream {
    client: Client,
    slot_name: String,
    publication_name: String,
    change_buffer: VecDeque<Change>,
    last_received_lsn: Option<String>,
    last_processed_lsn: Option<String>,
}

impl ReplicationStream {
    pub async fn new(
        connection_string: &str,
        slot_name: &str,
        publication_name: &str,
        create_slot: bool,
        _start_lsn: Option<String>,
    ) -> Result<Self> {
        // Parse connection string
        let config = connection_string.parse::<tokio_postgres::Config>()?;

        // Create a client
        let (client, connection) = config.connect(NoTls).await?;

        tokio::spawn(async move {
            if let Err(e) = connection.await {
                eprintln!("Connection error: {}", e);
            }
        });

        // Create replication slot if requested
        if create_slot {
            match Self::create_replication_slot(&client, slot_name).await {
                Ok(_) => eprintln!("Created replication slot: {}", slot_name),
                Err(e) => {
                    let err_msg = e.to_string().to_lowercase();
                    if err_msg.contains("already exists") || err_msg.contains("exist") {
                        eprintln!("Replication slot '{}' already exists, continuing...", slot_name);
                    } else {
                        return Err(e);
                    }
                }
            }
        }

        eprintln!("Starting replication stream...\n");

        Ok(Self {
            client,
            slot_name: slot_name.to_string(),
            publication_name: publication_name.to_string(),
            change_buffer: VecDeque::new(),
            last_received_lsn: None,
            last_processed_lsn: None,
        })
    }

    async fn create_replication_slot(client: &Client, slot_name: &str) -> Result<()> {
        // Use SQL function instead of replication protocol command
        let query = format!(
            "SELECT pg_create_logical_replication_slot('{}', 'pgoutput')",
            slot_name
        );
        
        let rows = client.simple_query(&query).await?;
        
        for row in rows {
            if let SimpleQueryMessage::Row(row) = row {
                eprintln!("Slot created: {:?}", row);
            }
        }
        
        Ok(())
    }

    pub async fn next_message(&mut self) -> Result<Option<Change>> {
        // If we have buffered changes, return the next one
        if let Some(change) = self.change_buffer.pop_front() {
            return Ok(Some(change));
        }

        // Poll for changes and buffer them
        loop {
            let query = format!(
                "SELECT lsn, xid, data FROM pg_logical_slot_get_binary_changes('{}', NULL, NULL, 'proto_version', '1', 'publication_names', '{}')",
                self.slot_name, self.publication_name
            );

            let rows = self.client.query(&query, &[]).await?;
            
            if rows.is_empty() {
                // No changes available, sleep briefly and retry
                tokio::time::sleep(Duration::from_millis(100)).await;
                continue;
            }

            // Process all rows and buffer the changes
            for row in rows {
                let lsn: String = row.get(0);
                let data: Vec<u8> = row.get(2);
                
                // Update last received LSN
                self.last_received_lsn = Some(lsn.clone());
                
                // Decode the pgoutput message
                if let Some(change) = decode_pgoutput_message(&data)? {
                    self.change_buffer.push_back(change);
                }
            }

            // Return the first buffered change
            if let Some(change) = self.change_buffer.pop_front() {
                return Ok(Some(change));
            }
        }
    }
    
    /// Mark an LSN as successfully processed
    /// Note: pg_logical_slot_get_binary_changes already auto-confirms,
    /// but this tracks progress for monitoring/debugging
    pub fn mark_processed(&mut self, lsn: &str) {
        self.last_processed_lsn = Some(lsn.to_string());
    }
    
    /// Get the last received LSN from PostgreSQL
    pub fn last_received_lsn(&self) -> Option<&str> {
        self.last_received_lsn.as_deref()
    }
    
    /// Get the last successfully processed LSN
    pub fn last_processed_lsn(&self) -> Option<&str> {
        self.last_processed_lsn.as_deref()
    }
    
    /// Get replication slot status from PostgreSQL
    pub async fn get_slot_status(&self) -> Result<SlotStatus> {
        let query = format!(
            "SELECT confirmed_flush_lsn, restart_lsn, active FROM pg_replication_slots WHERE slot_name = '{}'",
            self.slot_name
        );
        
        let rows = self.client.query(&query, &[]).await?;
        
        if rows.is_empty() {
            return Err(anyhow::anyhow!("Replication slot '{}' not found", self.slot_name));
        }
        
        let row = &rows[0];
        Ok(SlotStatus {
            confirmed_flush_lsn: row.get(0),
            restart_lsn: row.get(1),
            active: row.get(2),
        })
    }
}

#[derive(Debug, Clone)]
pub struct SlotStatus {
    pub confirmed_flush_lsn: String,
    pub restart_lsn: String,
    pub active: bool,
}
