use futures::FutureExt;
use homie::{Datatype, HomieDevice, Node, Property};
use rand::random;
use rumqttc::MqttOptions;
use std::error::Error;
use std::time::Duration;
use tokio::task::{self, JoinHandle};
use tokio::{time, try_join};

#[tokio::main(core_threads = 2)]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    pretty_env_logger::init();

    let mqttoptions = MqttOptions::new("homie_example", "test.mosquitto.org", 1883);

    let (mut homie, homie_handle) =
        HomieDevice::builder("homie/example", "Homie sensor example", mqttoptions)
            .spawn()
            .await?;

    homie
        .add_node(Node::new(
            "sensor".to_string(),
            "Sensor".to_string(),
            "Environment sensor".to_string(),
            vec![
                Property::new("temperature", "Temperature", Datatype::Float, Some("ºC")),
                Property::new("humidity", "Humidity", Datatype::Integer, Some("%")),
            ],
        ))
        .await?;

    let handle: JoinHandle<Result<(), Box<dyn Error + Send + Sync>>> = task::spawn(async move {
        homie.ready().await?;
        println!("Ready");

        loop {
            let temperature: f32 = random::<f32>() * 40.0;
            let humidity: u8 = (random::<f32>() * 100.0) as u8;
            println!("Update: {}ºC {}%", temperature, humidity);
            homie
                .publish_value("sensor", "temperature", temperature)
                .await?;
            homie.publish_value("sensor", "humidity", humidity).await?;

            time::delay_for(Duration::from_secs(10)).await;
        }
    });

    // Poll everything to completion, until the first one bombs out.
    let res: Result<_, Box<dyn Error + Send + Sync>> = try_join! {
        homie_handle,
        handle.map(|res| Ok(res??)),
    };
    res?;
    Ok(())
}
