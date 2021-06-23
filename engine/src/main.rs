use chainflip_engine::{
    eth,
    health::health_check,
    mq::nats_client::NatsMQClientFactory,
    p2p::ValidatorId,
    sc_observer,
    settings::Settings,
    signing::{self, crypto::Parameters},
};

#[tokio::main]
async fn main() {
    env_logger::init();

    log::info!("Start the engines! :broom: :broom: ");

    let settings = Settings::new().expect("Failed to initialise settings");

    // can use this sender to shut down the health check gracefully
    let _sender = health_check(settings.engine.health_check_port).await;

    sc_observer::sc_observer::start(settings.clone()).await;

    eth::start(settings.clone())
        .await
        .expect("Should start ETH client");

    let mq_factory = NatsMQClientFactory::new(&settings.message_queue);

    // TODO: read the key for config/file
    let signer_idx = ValidatorId("0".to_string());

    let params = Parameters {
        share_count: 150,
        threshold: 99,
    };

    let signing_client = signing::MultisigClient::new(mq_factory, signer_idx, params);

    signing_client.run().await;
}
