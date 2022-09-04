/// This is a subset of the full LDtk schema, which is at `ldtk_full_unused.rs`.
extern crate serde_derive;

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
}
