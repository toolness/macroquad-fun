/// This is a subset of the full LDtk schema, which is at `ldtk_full_unused.rs`.
/// We also add some convenience methods.
extern crate serde_derive;

use std::collections::HashMap;

use anyhow::{anyhow, Error, Result};
use macroquad::prelude::Vec2;
use serde::{Deserialize, Deserializer};
use uuid::Uuid;

#[derive(Deserialize)]
pub struct Coordinate {
    /// File format version
    #[serde(rename = "jsonVersion")]
    pub json_version: String,

    /// All levels. The order of this array is only relevant in `LinearHorizontal` and
    /// `linearVertical` world layouts (see `worldLayout` value).<br/>  Otherwise, you should
    /// refer to the `worldX`,`worldY` coordinates of each Level.
    pub levels: Vec<Level>,
}

#[derive(Deserialize)]
pub struct Level {
    /// User defined unique identifier
    pub identifier: String,

    /// Height of the level in pixels
    #[serde(rename = "pxHei")]
    pub px_hei: i64,

    /// Width of the level in pixels
    #[serde(rename = "pxWid")]
    pub px_wid: i64,

    /// World X coordinate in pixels.<br/>  Only relevant for world layouts where level spatial
    /// positioning is manual (ie. GridVania, Free). For Horizontal and Vertical layouts, the
    /// value is always -1 here.
    #[serde(rename = "worldX")]
    pub world_x: i64,

    /// World Y coordinate in pixels.<br/>  Only relevant for world layouts where level spatial
    /// positioning is manual (ie. GridVania, Free). For Horizontal and Vertical layouts, the
    /// value is always -1 here.
    #[serde(rename = "worldY")]
    pub world_y: i64,

    /// An array containing all Layer instances. **IMPORTANT**: if the project option "*Save
    /// levels separately*" is enabled, this field will be `null`.<br/>  This array is **sorted
    /// in display order**: the 1st layer is the top-most and the last is behind.
    #[serde(rename = "layerInstances")]
    pub layer_instances: Option<Vec<LayerInstance>>,
}

#[derive(Deserialize)]
pub struct LayerInstance {
    /// Layer definition identifier
    #[serde(rename = "__identifier")]
    pub identifier: String,

    /// Grid-based height
    #[serde(rename = "__cHei")]
    pub c_hei: i64,

    /// Grid-based width
    #[serde(rename = "__cWid")]
    pub c_wid: i64,

    /// Grid size
    #[serde(rename = "__gridSize")]
    pub grid_size: i64,

    /// A list of all values in the IntGrid layer, stored in CSV format (Comma Separated
    /// Values).<br/>  Order is from left to right, and top to bottom (ie. first row from left to
    /// right, followed by second row, etc).<br/>  `0` means "empty cell" and IntGrid values
    /// start at 1.<br/>  The array size is `__cWid` x `__cHei` cells.
    #[serde(rename = "intGridCsv")]
    pub int_grid_csv: Vec<i64>,

    #[serde(rename = "entityInstances")]
    pub entity_instances: Vec<EntityInstance>,

    /// An array containing all tiles generated by Auto-layer rules. The array is already sorted
    /// in display order (ie. 1st tile is beneath 2nd, which is beneath 3rd etc.).<br/><br/>
    /// Note: if multiple tiles are stacked in the same cell as the result of different rules,
    /// all tiles behind opaque ones will be discarded.
    #[serde(rename = "autoLayerTiles")]
    pub auto_layer_tiles: Vec<TileInstance>,

    #[serde(rename = "gridTiles")]
    pub grid_tiles: Vec<TileInstance>,
}

/// This structure represents a single tile from a given Tileset.
#[derive(Deserialize)]
pub struct TileInstance {
    /// "Flip bits", a 2-bits integer to represent the mirror transformations of the tile.<br/>
    /// - Bit 0 = X flip<br/>   - Bit 1 = Y flip<br/>   Examples: f=0 (no flip), f=1 (X flip
    /// only), f=2 (Y flip only), f=3 (both flips)
    #[serde(rename = "f")]
    pub flip_bits: i64,
    /// Pixel coordinates of the tile in the **layer** (`[x,y]` format). Don't forget optional
    /// layer offsets, if they exist!
    #[serde(rename = "px")]
    pub layer_px: Vec<i64>,
    /// Pixel coordinates of the tile in the **tileset** (`[x,y]` format)
    #[serde(rename = "src")]
    pub tileset_px: Vec<i64>,
    /// The *Tile ID* in the corresponding tileset.
    pub t: i64,
}

#[derive(Deserialize)]
pub struct EntityInstance {
    /// Grid-based coordinates (`[x,y]` format)
    #[serde(rename = "__grid")]
    pub grid: Vec<i64>,

    /// Entity definition identifier
    #[serde(rename = "__identifier")]
    pub identifier: String,

    /// Entity height in pixels. For non-resizable entities, it will be the same as Entity
    /// definition.
    pub height: i64,

    /// Unique instance identifier
    pub iid: Uuid,

    /// Pixel coordinates (`[x,y]` format) in current level coordinate space. Don't forget
    /// optional layer offsets, if they exist!
    pub px: Vec<i64>,

    /// Entity width in pixels. For non-resizable entities, it will be the same as Entity
    /// definition.
    pub width: i64,

    /// An array of all custom fields and their values.
    #[serde(rename = "fieldInstances", deserialize_with = "field_instance_hashmap")]
    pub field_instances: HashMap<String, FieldInstance>,
}

/// LDtk serializes field instances as an array, I'm not sure why. It's much more
/// convenient to consume them as a map, so we'll do that during deserialization.
fn field_instance_hashmap<'de, D>(
    deserializer: D,
) -> Result<HashMap<String, FieldInstance>, D::Error>
where
    D: Deserializer<'de>,
{
    let fields: Vec<FieldInstance> = Vec::deserialize(deserializer)?;
    let mut result: HashMap<String, FieldInstance> = HashMap::with_capacity(fields.len());

    for field in fields {
        if result.contains_key(&field.identifier) {
            return Err(serde::de::Error::custom(format!(
                "Multiple entries for '{}' found!",
                field.identifier
            )));
        }
        result.insert(field.identifier.clone(), field);
    }

    Ok(result)
}

pub fn field_into<T>(fields: &mut HashMap<String, FieldInstance>, name: &'static str) -> Result<T>
where
    T: TryFrom<FieldInstance, Error = Error>,
{
    let opt_field = fields.remove(name);
    if let Some(field) = opt_field {
        field.try_into()
    } else {
        Err(anyhow!(
            "Expected field instance with identifier '{}' to exist",
            name
        ))
    }
}

#[derive(Deserialize)]
pub struct FieldInstance {
    /// Field definition identifier
    #[serde(rename = "__identifier")]
    pub identifier: String,

    /// Type of the field, such as `Int`, `Float`, `String`, `Enum(my_enum_name)`, `Bool`,
    /// etc.<br/>  NOTE: if you enable the advanced option **Use Multilines type**, you will have
    /// "*Multilines*" instead of "*String*" when relevant.
    #[serde(rename = "__type")]
    pub field_instance_type: String,

    /// Actual value of the field instance. The value type varies, depending on `__type`:<br/>
    /// - For **classic types** (ie. Integer, Float, Boolean, String, Text and FilePath), you
    /// just get the actual value with the expected type.<br/>   - For **Color**, the value is an
    /// hexadecimal string using "#rrggbb" format.<br/>   - For **Enum**, the value is a String
    /// representing the selected enum value.<br/>   - For **Point**, the value is a
    /// [GridPoint](#ldtk-GridPoint) object.<br/>   - For **Tile**, the value is a
    /// [TilesetRect](#ldtk-TilesetRect) object.<br/>   - For **EntityRef**, the value is an
    /// [EntityReferenceInfos](#ldtk-EntityReferenceInfos) object.<br/><br/>  If the field is an
    /// array, then this `__value` will also be a JSON array.
    #[serde(rename = "__value")]
    pub value: Option<serde_json::Value>,
}

impl FieldInstance {
    pub fn value_result(self) -> Result<serde_json::Value> {
        if let Some(value) = self.value {
            Ok(value)
        } else {
            Err(anyhow!(
                "Expected field instance with identified '{}' to have a value",
                self.identifier
            ))
        }
    }
}

#[derive(Deserialize)]
pub struct Point {
    cx: f32,
    cy: f32,
}

impl From<Point> for Vec2 {
    fn from(point: Point) -> Self {
        Vec2::new(point.cx, point.cy)
    }
}

impl TryFrom<FieldInstance> for Vec2 {
    type Error = anyhow::Error;

    fn try_from(value: FieldInstance) -> Result<Self> {
        let point: Point = serde_json::from_value(value.value_result()?)?;
        Ok(point.into())
    }
}

#[derive(PartialEq, Deserialize)]
pub struct EntityRef {
    #[serde(rename = "entityIid")]
    pub iid: Uuid,
}

impl TryFrom<FieldInstance> for Option<EntityRef> {
    type Error = anyhow::Error;

    fn try_from(value: FieldInstance) -> Result<Self> {
        match value.value {
            Some(serde_json::Value::Null) => Ok(None),
            None => Ok(None),
            _ => Ok(Some(serde_json::from_value(value.value_result()?)?)),
        }
    }
}

/// TODO: This is the exact same thing as for Option<EntityRef>, we should be able
/// to make a more generic trait implementation here.
impl TryFrom<FieldInstance> for Option<String> {
    type Error = anyhow::Error;

    fn try_from(value: FieldInstance) -> Result<Self> {
        match value.value {
            Some(serde_json::Value::Null) => Ok(None),
            None => Ok(None),
            _ => Ok(Some(serde_json::from_value(value.value_result()?)?)),
        }
    }
}

impl TryFrom<FieldInstance> for bool {
    type Error = anyhow::Error;

    fn try_from(value: FieldInstance) -> Result<Self> {
        if let Some(serde_json::Value::Bool(value)) = value.value {
            return Ok(value);
        }
        Err(anyhow!(
            "Expected field instance with identifier '{}' to be a boolean",
            value.identifier
        ))
    }
}

impl TryFrom<FieldInstance> for f64 {
    type Error = anyhow::Error;

    fn try_from(value: FieldInstance) -> Result<Self> {
        if let Some(serde_json::Value::Number(value)) = value.value {
            if let Some(number) = value.as_f64() {
                return Ok(number);
            }
        }
        Err(anyhow!(
            "Expected field instance with identifier '{}' to be a number",
            value.identifier
        ))
    }
}

impl TryFrom<FieldInstance> for f32 {
    type Error = anyhow::Error;

    fn try_from(value: FieldInstance) -> Result<Self> {
        f64::try_from(value).map(|v| v as f32)
    }
}

impl TryFrom<FieldInstance> for String {
    type Error = anyhow::Error;

    fn try_from(value: FieldInstance) -> Result<Self> {
        if let Some(serde_json::Value::String(value)) = value.value {
            return Ok(value);
        }
        Err(anyhow!(
            "Expected field instance with identifier '{}' to be a string",
            value.identifier
        ))
    }
}
