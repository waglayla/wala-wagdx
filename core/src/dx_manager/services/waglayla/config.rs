use crate::app::{GIT_DESCRIBE, VERSION};
use crate::imports::*;
use crate::utils::Arglist;
use waglayla_core::waglaylad_env;
#[cfg(not(target_arch = "wasm32"))]
pub use waglaylad_lib::args::Args;

fn user_agent_comment() -> String {
    format!("waglayla-ng:{}-{}", VERSION, GIT_DESCRIBE)
}

#[allow(dead_code)]
fn user_agent() -> String {
    format!(
        "/{}:{}/waglayla-ng:{}-{}/",
        waglaylad_env::name(),
        waglaylad_env::version(),
        VERSION,
        GIT_DESCRIBE
    )
}

#[derive(Debug, Clone)]
pub struct Config {
    enable_upnp: bool,
    enable_wrpc_borsh: bool,
    #[allow(dead_code)]
    enable_wrpc_json: bool,
    enable_grpc: bool,
    grpc_network_interface: NetworkInterfaceConfig,
    waglaylad_daemon_args_enable: bool,
    waglaylad_daemon_args: String,
    waglaylad_daemon_storage_folder_enable: bool,
    waglaylad_daemon_storage_folder: String,
}

impl From<NodeSettings> for Config {
    fn from(node_settings: NodeSettings) -> Self {
        Self {
            enable_upnp: node_settings.enable_upnp,
            enable_wrpc_borsh: node_settings.enable_wrpc_borsh,
            enable_wrpc_json: node_settings.enable_wrpc_json,
            enable_grpc: node_settings.enable_grpc,
            grpc_network_interface: node_settings.grpc_network_interface,
            waglaylad_daemon_args_enable: node_settings.waglaylad_daemon_args_enable,
            waglaylad_daemon_args: node_settings.waglaylad_daemon_args,
            waglaylad_daemon_storage_folder_enable: node_settings.waglaylad_daemon_storage_folder_enable,
            waglaylad_daemon_storage_folder: node_settings.waglaylad_daemon_storage_folder,
        }
    }
}

cfg_if! {

    if #[cfg(not(target_arch = "wasm32"))] {
        impl TryFrom<Config> for Args {
            type Error = Error;
            fn try_from(config: Config) -> Result<Self> {
                let mut args = Args::default();

                args.perf_metrics = true;
                args.perf_metrics_interval_sec = 1;
                args.yes = true;
                args.utxoindex = true;
                args.disable_upnp = !config.enable_upnp;

                if config.enable_grpc {
                    args.rpclisten = Some(config.grpc_network_interface.into());
                }

                args.user_agent_comments = vec![user_agent_comment()];

                // TODO - parse custom args and overlap on top of the defaults

                Ok(args)
            }
        }

        impl From<Config> for Vec<String> {
            fn from(config: Config) -> Self {
                let mut args = Arglist::default();

                args.push("--perf-metrics");
                args.push("--perf-metrics-interval-sec=1");
                args.push("--yes");
                args.push("--utxoindex");
                args.push(format!("--ram-scale={:1.2}", 1.0));

                if !config.enable_upnp {
                    args.push("--disable-upnp");
                }

                if config.enable_grpc {
                    args.push(format!("--rpclisten={}", config.grpc_network_interface));
                } else {
                    args.push("--nogrpc");
                }

                if config.enable_wrpc_borsh {
                    args.push("--rpclisten-borsh=0.0.0.0:13110");
                } else {
                    args.push("--rpclisten-borsh=127.0.0.1:13110");
                }

                args.push(format!("--uacomment={}", user_agent_comment()));

                if config.waglaylad_daemon_storage_folder_enable && !config.waglaylad_daemon_storage_folder.is_empty() && !(config.waglaylad_daemon_args_enable && config.waglaylad_daemon_args.contains("--appdir")) {
                    args.push(format!("--appdir={}", config.waglaylad_daemon_storage_folder));
                }

                if config.waglaylad_daemon_args_enable {
                    config.waglaylad_daemon_args.trim().split(' ').filter(|arg|!arg.trim().is_empty()).for_each(|arg| {
                        args.push(arg);
                    });
                }

                args.into()
            }
        }

        impl IntoIterator for Config {
            type Item = String;
            type IntoIter = std::vec::IntoIter<Self::Item>;

            fn into_iter(self) -> Self::IntoIter {
                let args: Vec<String> = self.into();
                args.into_iter()
            }
        }
    }
}
