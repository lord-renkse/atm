use anyhow::Result;
use atm::processor::Processor;
use atm::reader;
use atm::reader::Reader;
use atm::writer;
use atm::writer::Writer;
use tokio::sync::mpsc;
use tokio::task::JoinHandle;

#[tokio::main]
async fn main() -> Result<()> {
    // the reason I chose mpsc over oneshot is that I leave open the possibility of having more senders for potential software extension
    let (sender_operations, receiver_operations) = mpsc::unbounded_channel::<reader::Command>();
    // the reason I chose mpsc over oneshot is that I leave open the possibility of having more senders for potential software extension
    let (sender_results, receiver_results) = mpsc::unbounded_channel::<writer::Command>();

    // create a task for the main processor
    let start_processor: JoinHandle<Result<()>> = tokio::spawn(async move {
        let mut processor = Processor::build(receiver_operations, sender_results);
        processor.run().await?;
        Ok(())
    });

    // create a task for the writer (receive results and write them thru the terminal)
    let start_writer: JoinHandle<Result<()>> = tokio::spawn(async move {
        let mut writer = Writer::build(receiver_results);
        writer.run().await?;
        Ok(())
    });

    // create a task for the CSV reader
    let start_reader: JoinHandle<Result<()>> = tokio::spawn(async move {
        let reader = Reader::build(sender_operations);
        reader.run().await?;
        Ok(())
    });

    // Wait for the tasks to finish and propagate the error if any
    // This should be done in a better wait: handle the errors properly
    start_reader.await?.expect("Reader task failed");
    start_processor.await?.expect("Processor task failed");
    start_writer.await?.expect("Writer task failed");
    Ok(())
}
