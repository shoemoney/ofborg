extern crate ofborg;
extern crate amqp;
extern crate env_logger;

extern crate hyper;
extern crate hubcaps;
extern crate hyper_native_tls;


use std::env;

use amqp::Basic;

use ofborg::config;
use ofborg::worker;
use ofborg::tasks;
use ofborg::easyamqp;
use ofborg::easyamqp::TypedWrappers;


fn main() {
    let cfg = config::load(env::args().nth(1).unwrap().as_ref());
    ofborg::setup_log();

    println!("Hello, world!");


    let mut session = easyamqp::session_from_config(&cfg.rabbitmq).unwrap();
    println!("Connected to rabbitmq");

    let mut channel = session.open_channel(1).unwrap();

    channel
        .declare_exchange(easyamqp::ExchangeConfig {
            exchange: "github-events",
            exchange_type: easyamqp::ExchangeType::Topic,
            passive: false,
            durable: true,
            auto_delete: false,
            no_wait: false,
            internal: false,
            arguments: None,
        })
        .unwrap();

    channel
        .declare_queue(easyamqp::QueueConfig {
            queue: "mass-rebuild-check-jobs".to_owned(),
            passive: false,
            durable: true,
            exclusive: false,
            auto_delete: false,
            no_wait: false,
            arguments: None,
        })
        .unwrap();

    channel
        .declare_queue(easyamqp::QueueConfig {
            queue: "mass-rebuild-check-inputs".to_owned(),
            passive: false,
            durable: true,
            exclusive: false,
            auto_delete: false,
            no_wait: false,
            arguments: None,
        })
        .unwrap();

    channel
        .bind_queue(easyamqp::BindQueueConfig {
            queue: "mass-rebuild-check-inputs",
            exchange: "github-events",
            routing_key: Some("pull_request.nixos/nixpkgs"),
            no_wait: false,
            arguments: None,
        })
        .unwrap();

    channel.basic_prefetch(1).unwrap();
    channel
        .consume(
            worker::new(tasks::evaluationfilter::EvaluationFilterWorker::new(
                cfg.acl(),
            )),
            easyamqp::ConsumeConfig {
                queue: "mass-rebuild-check-inputs",
                consumer_tag: &format!("{}-evaluation-filter", cfg.whoami()),
                no_local: false,
                no_ack: false,
                no_wait: false,
                exclusive: false,
                arguments: None,
            },
        )
        .unwrap();

    channel.start_consuming();

    println!("Finished consuming?");

    channel.close(200, "Bye").unwrap();
    println!("Closed the channel");
    session.close(200, "Good Bye");
    println!("Closed the session... EOF");
}
