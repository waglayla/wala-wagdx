use crate::imports::*;
use chrono::{Datelike, NaiveDate, Utc};

pub const STAT_POLLING_INTERVAL_SECONDS: u64 = 1; // 1 sec

pub enum StatMonitorEvents {
  Enable,
  Disable,
  Exit,
}

// TODO: get/store server info

// pulled from coinbase.rs in WagLayla Rusty
const SUBSIDY_BY_MONTH_TABLE: [u64; 270] = [
  45625737738, 41634158882, 37991784282, 34668063717, 31635119661, 28867513459, 26342031965, 24037492838, 21934566882, 20015615919, 18264544852, 16666666666, 15208579246, 13878052960, 12663928094, 11556021239, 10545039887, 9622504486, 8780677321, 8012497612, 7311522294, 6671871973, 6088181617, 5555555555, 5069526415, 
  4626017653, 4221309364, 3852007079, 3515013295, 3207501495, 2926892440, 2670832537, 2437174098, 2223957324, 2029393872, 1851851851, 1689842138, 1542005884, 1407103121, 1284002359, 1171671098, 1069167165, 975630813, 890277512, 812391366, 741319108, 676464624, 617283950, 563280712, 514001961, 
  469034373, 428000786, 390557032, 356389055, 325210271, 296759170, 270797122, 247106369, 225488208, 205761316, 187760237, 171333987, 156344791, 142666928, 130185677, 118796351, 108403423, 98919723, 90265707, 82368789, 75162736, 68587105, 62586745, 57111329, 52114930, 
  47555642, 43395225, 39598783, 36134474, 32973241, 30088569, 27456263, 25054245, 22862368, 20862248, 19037109, 17371643, 15851880, 14465075, 13199594, 12044824, 10991080, 10029523, 9152087, 8351415, 7620789, 6954082, 6345703, 5790547, 5283960, 
  4821691, 4399864, 4014941, 3663693, 3343174, 3050695, 2783805, 2540263, 2318027, 2115234, 1930182, 1761320, 1607230, 1466621, 1338313, 1221231, 1114391, 1016898, 927935, 846754, 772675, 705078, 643394, 587106, 535743, 
  488873, 446104, 407077, 371463, 338966, 309311, 282251, 257558, 235026, 214464, 195702, 178581, 162957, 148701, 135692, 123821, 112988, 103103, 94083, 85852, 78342, 71488, 65234, 59527, 54319, 
  49567, 45230, 41273, 37662, 34367, 31361, 28617, 26114, 23829, 21744, 19842, 18106, 16522, 15076, 13757, 12554, 11455, 10453, 9539, 8704, 7943, 7248, 6614, 6035, 5507, 
  5025, 4585, 4184, 3818, 3484, 3179, 2901, 2647, 2416, 2204, 2011, 1835, 1675, 1528, 1394, 1272, 1161, 1059, 967, 882, 805, 734, 670, 611, 558, 
  509, 464, 424, 387, 353, 322, 294, 268, 244, 223, 203, 186, 169, 154, 141, 129, 117, 107, 98, 89, 81, 74, 67, 62, 56, 
  51, 47, 43, 39, 35, 32, 29, 27, 24, 22, 20, 18, 17, 15, 14, 13, 11, 10, 9, 9, 8, 7, 6, 6, 5, 
  5, 4, 4, 3, 3, 3, 3, 2, 2, 2, 2, 1, 1, 1, 1, 1, 1, 1, 1, 0,
];

fn get_block_reward() -> u64 {
  // Define the cutoff date: January 15th, 2025
  let cutoff_date = NaiveDate::from_ymd_opt(2025, 1, 15).unwrap();

  let now = Utc::now().naive_utc().date();
  if now < cutoff_date {
    50_000_000_000
  } else {
    let months_since_cutoff = (now.year() - cutoff_date.year()) * 12 + now.month() as i32 - cutoff_date.month() as i32;

    if months_since_cutoff >= 0 && months_since_cutoff < SUBSIDY_BY_MONTH_TABLE.len() as i32 {
      SUBSIDY_BY_MONTH_TABLE[months_since_cutoff as usize]
    } else {
      0
    }
  }
}

pub struct StatMonitorService {
  pub application_events: ApplicationEventsChannel,
  pub service_events: Channel<StatMonitorEvents>,
  pub task_ctl: Channel<()>,
  pub rpc_api: Mutex<Option<Arc<dyn RpcApi>>>,
  pub is_enabled: Arc<AtomicBool>,
}

impl StatMonitorService {
  pub fn new(application_events: ApplicationEventsChannel, _settings: &Settings) -> Self {
    Self {
      application_events,
      service_events: Channel::unbounded(),
      task_ctl: Channel::oneshot(),
      rpc_api: Mutex::new(None),
      is_enabled: Arc::new(AtomicBool::new(true)),
    }
  }

  pub fn rpc_api(&self) -> Option<Arc<dyn RpcApi>> {
    self.rpc_api.lock().unwrap().clone()
  }

  pub fn enable(&self) {
    self.service_events
      .sender
      .try_send(StatMonitorEvents::Enable)
      .unwrap();
  }

  pub fn disable(&self) {
    self.service_events
      .sender
      .try_send(StatMonitorEvents::Disable)
      .unwrap();
  }
}

#[async_trait]
impl Service for StatMonitorService {
  fn name(&self) -> &'static str {
    "stat-monitor"
  }

  async fn rpc_attach(self: Arc<Self>, rpc_api: &Arc<dyn RpcApi>) -> Result<()> {
    self.rpc_api.lock().unwrap().replace(rpc_api.clone());
    Ok(())
  }

  async fn rpc_detach(self: Arc<Self>) -> Result<()> {
    self.rpc_api.lock().unwrap().take();
    Ok(())
  }

  async fn launch(self: Arc<Self>) -> Result<()> {
    let this = self.clone();
    let _application_events_sender = self.application_events.sender.clone();

    let interval = task::interval(std::time::Duration::from_secs(STAT_POLLING_INTERVAL_SECONDS));
    pin_mut!(interval);

    loop {
      select! {
        _ = interval.next().fuse() => {
          if !self.is_enabled.load(Ordering::Relaxed) {
            continue;
          }

          let reward = get_block_reward();
          if let Err(e) = this.application_events.sender.try_send(
            Events::BlockRewardUpdate(reward)
          ) {
            log_error!("Failed to send block reward update: {}", e);
          }

          if let Some(rpc_api) = this.rpc_api() {
            if let Ok(resp) = rpc_api.get_coin_supply().await {
              let current_supply = resp.circulating_sompi;
              let max_supply = resp.max_sompi;

              if let Err(e) = this.application_events.sender.try_send(
                Events::CoinSupplyUpdate(current_supply, max_supply)
              ) {
                log_error!("Failed to send coin supply update: {}", e);
              }
            }

            if let Ok(resp) = rpc_api.estimate_network_hashes_per_second(1000, None).await {
              let hashes = resp;

              if let Err(e) = this.application_events.sender.try_send(
                Events::HashrateUpdate(hashes)
              ) {
                log_error!("Failed to send hashrate update: {}", e);
              }
            }

            if let Ok(resp) = rpc_api.get_block_dag_info().await {
              let diff = resp.difficulty;

              if let Err(e) = this.application_events.sender.try_send(
                Events::DifficultyUpdate(diff as u64)
              ) {
                log_error!("Failed to send difficulty update: {}", e);
              }
            }

            if let Ok(resp) = rpc_api.get_mempool_entries(false, false).await {
              let entries = resp.len();

              if let Err(e) = this.application_events.sender.try_send(
                Events::MempoolUpdate(entries)
              ) {
                log_error!("Failed to send mempool update: {}", e);
              }
            }
          }
        },
        msg = this.as_ref().service_events.receiver.recv().fuse() => {
          if let Ok(event) = msg {
            match event {
              StatMonitorEvents::Enable => {
                self.is_enabled.store(true, Ordering::Relaxed);
              }
              StatMonitorEvents::Disable => {
                self.is_enabled.store(false, Ordering::Relaxed);
              }
              StatMonitorEvents::Exit => {
                break;
              }
            }
          } else {
            break;
          }
        }
      }
    }

    this.task_ctl.send(()).await.unwrap();
    Ok(())
  }

  fn terminate(self: Arc<Self>) {
    self.service_events
      .sender
      .try_send(StatMonitorEvents::Exit)
      .unwrap();
  }

  async fn join(self: Arc<Self>) -> Result<()> {
    self.task_ctl.recv().await.unwrap();
    Ok(())
  }
}
