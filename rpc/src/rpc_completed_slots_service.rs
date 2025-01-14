use {
    crate::rpc_subscriptions::RpcSubscriptions,
    crossbeam_channel::RecvTimeoutError,
    safecoin_client::rpc_response::SlotUpdate,
    solana_ledger::blockstore::CompletedSlotsReceiver,
    safecoin_sdk::timing::timestamp,
    std::{
        sync::{
            atomic::{AtomicBool, Ordering},
            Arc,
        },
        thread::{Builder, JoinHandle},
        time::Duration,
    },
};

pub const COMPLETE_SLOT_REPORT_SLEEP_MS: u64 = 100;

pub struct RpcCompletedSlotsService;
impl RpcCompletedSlotsService {
    pub fn spawn(
        completed_slots_receiver: CompletedSlotsReceiver,
        rpc_subscriptions: Arc<RpcSubscriptions>,
        exit: Arc<AtomicBool>,
    ) -> JoinHandle<()> {
        Builder::new()
            .name("solana-rpc-completed-slots-service".to_string())
            .spawn(move || loop {
                // received exit signal, shutdown the service
                if exit.load(Ordering::Relaxed) {
                    break;
                }

                match completed_slots_receiver
                    .recv_timeout(Duration::from_millis(COMPLETE_SLOT_REPORT_SLEEP_MS))
                {
                    Err(RecvTimeoutError::Timeout) => {}
                    Err(RecvTimeoutError::Disconnected) => {
                        info!("RpcCompletedSlotService channel disconnected, exiting.");
                        break;
                    }
                    Ok(slots) => {
                        for slot in slots {
                            rpc_subscriptions.notify_slot_update(SlotUpdate::Completed {
                                slot,
                                timestamp: timestamp(),
                            });
                        }
                    }
                }
            })
            .unwrap()
    }
}
