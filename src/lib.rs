#![deny(clippy::all)]

use geozero::mvt::{tile, Message, Tile, TileValue};
use geozero::{geojson::GeoJson, wkb::Ewkb, GeozeroGeometry, ToGeo, ToMvt, ToWkb};
use napi::bindgen_prelude::Buffer;

#[macro_use]
extern crate napi_derive;

fn add_feature_attribute(
  mvt_layer: &mut tile::Layer,
  mvt_feature: &mut tile::Feature,
  key: String,
  value: TileValue,
) {
  // https://github.com/georust/geozero/blob/main/geozero/src/mvt/mvt_writer.rs

  let mvt_value = value.into();
  let key_entry = mvt_layer.keys.iter().position(|k| *k == key);
  // Optimization: maintain a hash table with key/index pairs
  let key_idx = match key_entry {
    None => {
      mvt_layer.keys.push(key);
      mvt_layer.keys.len() - 1
    }
    Some(idx) => idx,
  };
  mvt_feature.tags.push(key_idx as u32);

  let val_entry = mvt_layer.values.iter().position(|v| *v == mvt_value);
  // Optimization: maintain a hash table with value/index pairs
  let validx = match val_entry {
    None => {
      mvt_layer.values.push(mvt_value);
      mvt_layer.values.len() - 1
    }
    Some(idx) => idx,
  };
  mvt_feature.tags.push(validx as u32);
}

#[napi]
pub fn wkb2mvt(buf: Buffer, left: f64, bottom: f64, right: f64, top: f64, extent: u32) -> Buffer {
  let mut mvt_tile = Tile::default();

  let mut mvt_layer = tile::Layer {
    version: 2,
    ..Default::default()
  };
  mvt_layer.name = String::from("polygon");
  mvt_layer.extent = Some(4096);

  let _wkb = Ewkb(buf.to_vec());
  let feature = _wkb.to_mvt(extent, left, bottom, right, top).unwrap();
  mvt_layer.features.push(feature);
  mvt_tile.layers.push(mvt_layer);

  mvt_layer = tile::Layer {
    version: 2,
    ..Default::default()
  };
  mvt_layer.name = String::from("points");
  mvt_layer.extent = Some(4096);

  let mut mvt_feature = tile::Feature {
    id: Some(1),
    ..Default::default()
  };
  mvt_feature.set_type(tile::GeomType::Point);
  mvt_feature.geometry = [9, 4900, 6262].to_vec();

  add_feature_attribute(
    &mut mvt_layer,
    &mut mvt_feature,
    String::from("hello"),
    TileValue::Str("world".to_string()),
  );
  add_feature_attribute(
    &mut mvt_layer,
    &mut mvt_feature,
    String::from("h"),
    TileValue::Str("world".to_string()),
  );
  add_feature_attribute(
    &mut mvt_layer,
    &mut mvt_feature,
    String::from("count"),
    TileValue::Double(1.23),
  );

  mvt_layer.features.push(mvt_feature);

  mvt_feature = tile::Feature::default();
  mvt_feature.id = Some(2);
  mvt_feature.set_type(tile::GeomType::Point);
  mvt_feature.geometry = [9, 490, 6262].to_vec();

  add_feature_attribute(
    &mut mvt_layer,
    &mut mvt_feature,
    String::from("hello"),
    TileValue::Str("again".to_string()),
  );
  add_feature_attribute(
    &mut mvt_layer,
    &mut mvt_feature,
    String::from("count"),
    TileValue::Int(2),
  );

  mvt_layer.features.push(mvt_feature);
  mvt_tile.layers.push(mvt_layer);

  Buffer::from(mvt_tile.encode_to_vec())
}
