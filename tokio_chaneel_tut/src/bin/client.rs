use bytes::Bytes;
use mini_redis::client;
use tokio::sync::mpsc;
use tokio::sync::oneshot;

// Multiple dirrerent commands are multiplexed over a single channel
#[derive(Debug)]
enum Command {
    Get {
        key: String,
        resp: Responder<Option<Bytes>>,
    },
    Set {
        key: String,
        val: Bytes,
        resp: Responder<()>,
    },
}
// Provided by the requester and used by the manger task to send the command
// response back to the requester.
type Responder<T> = oneshot::Sender<mini_redis::Result<T>>;

#[tokio::main]

async fn main() {
    //Establish a connection to the server
    // The `move` keyword is used to *move* ownership of rx into the task.
    let (tx, mut rx) = mpsc::channel(32);

    let tx2 = tx.clone();

    // Spawn two tasks, one gets a key, the other sets a key
    let t1 = tokio::spawn(async move {
        let (resp_tx, resp_rx) = oneshot::channel();
        let cmd = Command::Get {
            key: "foo".to_string(),
            resp: resp_tx,
        };
        // Send the GET request
        // tx.send(cmd).await.unwrap();
        if tx.send(cmd).await.is_err(){
            eprint!("Connection task shutdown");
            return;
        }
        // Await the response
        let res = resp_rx.await;
        println!("GOT response tx {:?}", res);
    });

    let t2 = tokio::spawn(async move {
        let (resp_tx, resp_rx) = oneshot::channel();
        let cmd = Command::Set {
            key: "foo".to_string(),
            val: "bar".into(),
            resp: resp_tx,
        };
        // Send the Set request
        // tx2.send(cmd).await.unwrap();
        if tx2.send(cmd).await.is_err(){
            eprint!("connection task shutdown");
            return;
        }


        //Await for the response
        let res = resp_rx.await;
        println!("GOT response from tx2 {:?}",res)
    });

    let manager = tokio::spawn(async move {
        // Establish a connection to the server
        let mut client = client::connect("127.0.0.1:8080").await.unwrap();
        // start receiving messages
        while let Some(cmd) = rx.recv().await {
            use Command::*;

            match cmd {
                Get { key ,resp} => {
                    let res = client.get(&key).await;
                    let _ = resp.send(res);
                }
                Set { key, val ,resp} => {
                    let res = client.set(&key, val).await;

                    //Ignore errors
                    let _ = resp.send(res);
                }
            }
        }
    });

    //  let mut client = client::connect("127.0.0.1:8080").await.unwrap();
    // Spawn tow tasks, one gets a key, the other sets a key
    // Create a new channel with a capacity of at most 32.
    // creating one clone of tx(transmitter)
    //  let tx2 = tx.clone();
    //  tokio::spawn(async move {
    //     tx.send("sending from the first handle").await;
    //  });

    //  tokio::spawn(async move {
    //     tx2.send("sending from second handle").await;
    //  });

    //  while let Some(message) = rx.recv().await  {
    //      println!("GOT MESSAGE = {}", message);
    //  }

    //  let t1 = tokio::spawn(async move {
    //     let res = client.get("foo").await;
    //  });
    // client is moved here because client exist in current function.

    //  let t2 = tokio::spawn(async {
    //     client.set("foo", "bar".into()).await;
    //  });

    //  t1.await.unwrap();
    //  t2.await.unwrap();

    t1.await.unwrap();
    t2.await.unwrap();
    manager.await.unwrap();
}
