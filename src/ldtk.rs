/// This is a subset of the full LDtk schema, which is at `ldtk_full_unused.rs`.
/// We also add some convenience methods.
extern crate serde_derive;

use anyhow::{anyhow, Result};

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
}

#[derive(Serialize, Deserialize)]
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
    pub iid: String,

    /// Pixel coordinates (`[x,y]` format) in current level coordinate space. Don't forget
    /// optional layer offsets, if they exist!
    pub px: Vec<i64>,

    /// Entity width in pixels. For non-resizable entities, it will be the same as Entity
    /// definition.
    pub width: i64,

    /// An array of all custom fields and their values.
    #[serde(rename = "fieldInstances")]
    pub field_instances: Vec<FieldInstance>,
}

impl EntityInstance {
    pub fn get_string_field_instance(&self, identifier: &str) -> Result<String> {
        for field in self.field_instances.iter() {
            if field.identifier == identifier {
                if let Some(serde_json::Value::String(value)) = &field.value {
                    return Ok(value.to_owned());
                }
                return Err(anyhow!(
                    "Expected field instance with identifier '{}' to be a string",
                    identifier
                ));
            }
        }

        Err(anyhow!(
            "Unable to find field instance with identifier '{}'",
            identifier
        ))
    }
}

#[derive(Serialize, Deserialize)]
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
