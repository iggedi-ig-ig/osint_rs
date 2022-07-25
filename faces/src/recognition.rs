use image::imageops::FilterType;
use image::{DynamicImage, Rgb};
use lazy_static::lazy_static;
use nalgebra::DVector;
use tensorflow::{Graph, ImportGraphDefOptions, Session, SessionOptions, SessionRunArgs, Tensor};

const MODEL: &[u8] = include_bytes!("models/20180402-114759.pb");

lazy_static! {
    static ref GRAPH: Graph = {
        let mut graph = Graph::new();
        graph
            .import_graph_def(&*MODEL, &ImportGraphDefOptions::new())
            .unwrap();
        graph
    };
}

/// returns the embedding vector -> the result of the dimensionality reduction
pub fn get_embedding_vector(
    face_image: &DynamicImage,
) -> Result<DVector<f32>, Box<dyn std::error::Error + Sync + Send>> {
    let image = face_image
        .resize_exact(160, 160, FilterType::Lanczos3)
        .into_rgb32f();

    let flattened = image
        .pixels()
        // TODO: std might need to be 2 not 1
        .flat_map(|&Rgb([r, g, b])| [r, g, b].map(|f| (f - 0.5) * 2.0))
        .collect::<Vec<_>>();

    let input_tensor = Tensor::<f32>::new(&[1, 160, 160, 3]).with_values(&flattened)?;
    let phase_train = Tensor::new(&[]).with_values(&[false])?;

    let mut args = SessionRunArgs::new();
    let session = Session::new(&SessionOptions::new(), &GRAPH)?;

    args.add_feed(
        &GRAPH.operation_by_name_required("input")?,
        0,
        &input_tensor,
    );
    args.add_feed(
        &GRAPH.operation_by_name_required("phase_train")?,
        0,
        &phase_train,
    );

    let embedding = args.request_fetch(&GRAPH.operation_by_name_required("embeddings")?, 0);
    session.run(&mut args)?;

    Ok(DVector::from_vec(args.fetch::<f32>(embedding)?.to_vec()))
}
