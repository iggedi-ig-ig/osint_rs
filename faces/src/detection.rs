use image::{DynamicImage, GenericImageView};
use lazy_static::lazy_static;
use tensorflow::{Graph, ImportGraphDefOptions, Session, SessionOptions, SessionRunArgs, Tensor};

const MODEL: &[u8] = include_bytes!("models/mtcnn.pb");

// neural network configuration
const MIN_SIZE: f32 = 50.0;
const THRESHOLDS: &[f32] = &[0.6, 0.7, 0.7];
const FACTOR: f32 = 0.709;

lazy_static! {
    static ref GRAPH: Graph = {
        //Then we create a tensorflow graph from the model
        let mut graph = Graph::new();
        graph.import_graph_def(&*MODEL, &ImportGraphDefOptions::new()).unwrap();
        graph
    };
}

#[derive(Copy, Clone, Debug)]
// Make it a bit nicer to work with the results, by adding a more explanatory struct
pub struct BBox {
    pub x1: f32,
    pub y1: f32,
    pub x2: f32,
    pub y2: f32,
    pub prob: f32,
}

/// returns the bounding boxes of all faces in this image
pub fn find_face_bounding_boxes(
    input_image: &DynamicImage,
) -> Result<Vec<BBox>, Box<dyn std::error::Error + Sync + Send>> {
    let mut flattened: Vec<f32> = Vec::new();

    for (_x, _y, rgb) in input_image.pixels() {
        flattened.push(rgb[2] as f32);
        flattened.push(rgb[1] as f32);
        flattened.push(rgb[0] as f32);
    }

    //The `input` tensor expects BGR pixel data.
    let input = Tensor::new(&[input_image.height() as u64, input_image.width() as u64, 3])
        .with_values(&flattened)?;

    //Use input params from the existing module
    let min_size = Tensor::new(&[]).with_values(&[MIN_SIZE])?;
    let thresholds = Tensor::new(&[3]).with_values(&THRESHOLDS)?;
    let factor = Tensor::new(&[]).with_values(&[FACTOR])?;

    let mut args = SessionRunArgs::new();

    //Load default parameters
    args.add_feed(&GRAPH.operation_by_name_required("min_size")?, 0, &min_size);
    args.add_feed(
        &GRAPH.operation_by_name_required("thresholds")?,
        0,
        &thresholds,
    );
    args.add_feed(&GRAPH.operation_by_name_required("factor")?, 0, &factor);

    //Load our input image
    args.add_feed(&GRAPH.operation_by_name_required("input")?, 0, &input);

    //Request the following outputs after the session runs
    let bbox = args.request_fetch(&GRAPH.operation_by_name_required("box")?, 0);
    let prob = args.request_fetch(&GRAPH.operation_by_name_required("prob")?, 0);

    let session = Session::new(&SessionOptions::new(), &GRAPH)?;

    session.run(&mut args)?;

    //Our bounding box extents
    let bbox_res: Tensor<f32> = args.fetch(bbox)?;
    //Our facial probability
    let prob_res: Tensor<f32> = args.fetch(prob)?;

    Ok(bbox_res
        .chunks_exact(4) // Split into chunks of 4
        .zip(prob_res.iter()) // Combine it with prob_res
        .map(|(bbox, &prob)| BBox {
            y1: bbox[0],
            x1: bbox[1],
            y2: bbox[2],
            x2: bbox[3],
            prob,
        })
        .collect::<Vec<_>>())
}

/// returns all faces as images in the given image
pub fn get_extracted_faces(
    image: &DynamicImage,
) -> Result<Vec<(DynamicImage, BBox)>, Box<dyn std::error::Error + Sync + Send>> {
    find_face_bounding_boxes(&image).map(|faces| {
        faces
            .iter()
            .filter_map(|face| {
                let x = face.x1 as u32;
                let y = face.y1 as u32;

                let width = (face.x2 - face.x1).abs() as u32;
                let height = (face.y2 - face.y1).abs() as u32;

                if width * height > 0 {
                    let sub_image = image::imageops::crop_imm(image, x, y, width, height);
                    Some((DynamicImage::from(sub_image.to_image()), *face))
                } else {
                    None
                }
            })
            .collect()
    })
}
