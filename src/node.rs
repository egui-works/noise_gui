use {
    super::expr::{
        BlendExpr, ClampExpr, ControlPointExpr, CurveExpr, ExponentExpr, Expr, FractalExpr,
        RigidFractalExpr, ScaleBiasExpr, SelectExpr, TerraceExpr, WorleyExpr,
    },
    egui::TextureHandle,
    egui_snarl::Snarl,
    noise::{
        BasicMulti as Fractal, Cylinders, Perlin as AnySeedable, RidgedMulti as RigidFractal,
        Worley,
    },
    serde::{Deserialize, Serialize},
    std::collections::HashSet,
};

#[derive(Clone, Default, Serialize, Deserialize)]
pub struct BlendNode {
    pub image: Image,

    pub input_node_indices: [Option<usize>; 2],
    pub control_node_idx: Option<usize>,
    pub output_node_indices: HashSet<usize>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct CheckerboardNode {
    pub image: Image,

    pub output_node_indices: HashSet<usize>,

    pub size: NodeValue<u32>,
}

impl Default for CheckerboardNode {
    fn default() -> Self {
        Self {
            image: Default::default(),
            output_node_indices: Default::default(),
            size: NodeValue::Value(0), // TODO: Checkerboard::DEFAULT_SIZE is private!
        }
    }
}

#[derive(Clone, Default, Serialize, Deserialize)]
pub struct ClampNode {
    pub image: Image,

    pub input_node_idx: Option<usize>,
    pub output_node_indices: HashSet<usize>,

    pub lower_bound: NodeValue<f64>,
    pub upper_bound: NodeValue<f64>,
}

#[derive(Clone, Default, Serialize, Deserialize)]
pub struct CombinerNode {
    pub image: Image,

    pub input_node_indices: [Option<usize>; 2],
    pub output_node_indices: HashSet<usize>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct ConstantNode<T> {
    pub name: String,

    pub output_node_indices: HashSet<usize>,

    pub value: T,
}

impl<T> Default for ConstantNode<T>
where
    T: Default,
{
    fn default() -> Self {
        Self {
            name: "value".to_owned(),
            output_node_indices: Default::default(),
            value: Default::default(),
        }
    }
}

#[derive(Clone, Default, Serialize, Deserialize)]
pub struct ControlPointNode {
    pub output_node_indices: HashSet<usize>,

    pub input: NodeValue<f64>,
    pub output: NodeValue<f64>,
}

#[derive(Clone, Default, Serialize, Deserialize)]
pub struct CurveNode {
    pub image: Image,

    pub input_node_idx: Option<usize>,
    pub output_node_indices: HashSet<usize>,

    pub control_point_node_indices: Vec<Option<usize>>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct CylindersNode {
    pub image: Image,

    pub output_node_indices: HashSet<usize>,

    pub frequency: NodeValue<f64>,
}

impl Default for CylindersNode {
    fn default() -> Self {
        Self {
            image: Default::default(),
            output_node_indices: Default::default(),
            frequency: NodeValue::Value(Cylinders::DEFAULT_FREQUENCY),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum DistanceFunction {
    Chebyshev,
    Euclidean,
    EuclideanSquared,
    Manhattan,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct ExponentNode {
    pub image: Image,

    pub input_node_idx: Option<usize>,
    pub output_node_indices: HashSet<usize>,

    pub exponent: NodeValue<f64>,
}

impl Default for ExponentNode {
    fn default() -> Self {
        Self {
            image: Default::default(),
            input_node_idx: Default::default(),
            output_node_indices: Default::default(),
            exponent: NodeValue::Value(1.0),
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct FractalNode {
    pub image: Image,

    pub output_node_indices: HashSet<usize>,

    pub source: Source,
    pub seed: NodeValue<u32>,
    pub octaves: NodeValue<u32>,
    pub frequency: NodeValue<f64>,
    pub lacunarity: NodeValue<f64>,
    pub persistence: NodeValue<f64>,
}

impl FractalNode {
    pub const MAX_OCTAVES: u32 = Fractal::<AnySeedable>::MAX_OCTAVES as _;
}

impl Default for FractalNode {
    fn default() -> Self {
        Self {
            image: Default::default(),
            output_node_indices: Default::default(),
            source: Default::default(),
            seed: NodeValue::Value(Fractal::<AnySeedable>::DEFAULT_SEED),
            octaves: NodeValue::Value(Fractal::<AnySeedable>::DEFAULT_OCTAVES as _),
            frequency: NodeValue::Value(Fractal::<AnySeedable>::DEFAULT_FREQUENCY),
            lacunarity: NodeValue::Value(Fractal::<AnySeedable>::DEFAULT_LACUNARITY),
            persistence: NodeValue::Value(Fractal::<AnySeedable>::DEFAULT_PERSISTENCE),
        }
    }
}

#[derive(Clone, Default, Serialize, Deserialize)]
pub struct GeneratorNode {
    pub image: Image,

    pub output_node_indices: HashSet<usize>,

    pub seed: NodeValue<u32>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Image {
    pub scale: f64,

    #[serde(skip)]
    pub texture: Option<TextureHandle>,

    #[serde(skip)]
    pub version: usize,

    pub x: f64,
    pub y: f64,
}

impl Default for Image {
    fn default() -> Self {
        Self {
            scale: 4.0,
            texture: None,
            version: 0,
            x: 0.0,
            y: 0.0,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum NodeValue<T> {
    Node(usize),
    Value(T),
}

impl<T> NodeValue<T> {
    pub fn as_node_index(&self) -> Option<usize> {
        if let &Self::Node(node_idx) = self {
            Some(node_idx)
        } else {
            None
        }
    }

    // pub fn as_value(&self) -> Option<&T> {
    //     if let Self::Value(value) = self {
    //         Some(value)
    //     } else {
    //         None
    //     }
    // }

    pub fn as_value_mut(&mut self) -> Option<&mut T> {
        if let Self::Value(value) = self {
            Some(value)
        } else {
            None
        }
    }
}

impl NodeValue<f64> {
    fn value(self, snarl: &Snarl<NoiseNode>) -> f64 {
        match self {
            Self::Node(node_idx) => snarl.get_node(node_idx).as_const_f64().unwrap(),
            Self::Value(value) => value,
        }
    }
}

impl NodeValue<u32> {
    fn value(self, snarl: &Snarl<NoiseNode>) -> u32 {
        match self {
            Self::Node(node_idx) => snarl.get_node(node_idx).as_const_u32().unwrap(),
            Self::Value(value) => value,
        }
    }
}

impl<T> Default for NodeValue<T>
where
    T: Default,
{
    fn default() -> Self {
        Self::Value(Default::default())
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub enum NoiseNode {
    Abs(UnaryNode),
    Add(CombinerNode),
    BasicMulti(FractalNode),
    Billow(FractalNode),
    Blend(BlendNode),
    Clamp(ClampNode),
    Checkerboard(CheckerboardNode),
    ControlPoint(ControlPointNode),
    Curve(CurveNode),
    Cylinders(CylindersNode),
    Exponent(ExponentNode),
    F64(ConstantNode<f64>),
    Fbm(FractalNode),
    HybridMulti(FractalNode),
    Max(CombinerNode),
    Min(CombinerNode),
    Multiply(CombinerNode),
    Negate(UnaryNode),
    OpenSimplex(GeneratorNode),
    Perlin(GeneratorNode),
    PerlinSurflet(GeneratorNode),
    Power(CombinerNode),
    RigidMulti(RigidFractalNode),
    ScaleBias(ScaleBiasNode),
    Select(SelectNode),
    Simplex(GeneratorNode),
    SuperSimplex(GeneratorNode),
    Terrace(TerraceNode),
    U32(ConstantNode<u32>),
    Value(GeneratorNode),
    Worley(WorleyNode),
}

impl NoiseNode {
    pub fn as_blend_mut(&mut self) -> Option<&mut BlendNode> {
        if let Self::Blend(node) = self {
            Some(node)
        } else {
            None
        }
    }

    pub fn as_checkerboard_mut(&mut self) -> Option<&mut CheckerboardNode> {
        if let Self::Checkerboard(node) = self {
            Some(node)
        } else {
            None
        }
    }

    pub fn as_clamp_mut(&mut self) -> Option<&mut ClampNode> {
        if let Self::Clamp(node) = self {
            Some(node)
        } else {
            None
        }
    }

    pub fn as_combiner_mut(&mut self) -> Option<&mut CombinerNode> {
        if let Self::Add(node)
        | Self::Max(node)
        | Self::Min(node)
        | Self::Multiply(node)
        | Self::Power(node) = self
        {
            Some(node)
        } else {
            None
        }
    }

    pub fn as_const_f64(&self) -> Option<f64> {
        if let &Self::F64(ConstantNode { value, .. }) = self {
            Some(value)
        } else {
            None
        }
    }

    pub fn as_const_u32(&self) -> Option<u32> {
        if let &Self::U32(ConstantNode { value, .. }) = self {
            Some(value)
        } else {
            None
        }
    }

    pub fn as_control_point(&self) -> Option<&ControlPointNode> {
        if let Self::ControlPoint(node) = self {
            Some(node)
        } else {
            None
        }
    }

    pub fn as_control_point_mut(&mut self) -> Option<&mut ControlPointNode> {
        if let Self::ControlPoint(node) = self {
            Some(node)
        } else {
            None
        }
    }

    pub fn as_curve_mut(&mut self) -> Option<&mut CurveNode> {
        if let Self::Curve(node) = self {
            Some(node)
        } else {
            None
        }
    }

    pub fn as_cylinders_mut(&mut self) -> Option<&mut CylindersNode> {
        if let Self::Cylinders(node) = self {
            Some(node)
        } else {
            None
        }
    }

    pub fn as_exponent_mut(&mut self) -> Option<&mut ExponentNode> {
        if let Self::Exponent(node) = self {
            Some(node)
        } else {
            None
        }
    }

    pub fn as_fractal_mut(&mut self) -> Option<&mut FractalNode> {
        if let Self::BasicMulti(node)
        | Self::Billow(node)
        | Self::Fbm(node)
        | Self::HybridMulti(node) = self
        {
            Some(node)
        } else {
            None
        }
    }

    pub fn as_generator_mut(&mut self) -> Option<&mut GeneratorNode> {
        if let Self::OpenSimplex(node)
        | Self::Perlin(node)
        | Self::PerlinSurflet(node)
        | Self::Simplex(node)
        | Self::SuperSimplex(node)
        | Self::Value(node) = self
        {
            Some(node)
        } else {
            None
        }
    }

    pub fn as_rigid_fractal_mut(&mut self) -> Option<&mut RigidFractalNode> {
        if let Self::RigidMulti(node) = self {
            Some(node)
        } else {
            None
        }
    }

    pub fn as_scale_bias_mut(&mut self) -> Option<&mut ScaleBiasNode> {
        if let Self::ScaleBias(node) = self {
            Some(node)
        } else {
            None
        }
    }

    pub fn as_select_mut(&mut self) -> Option<&mut SelectNode> {
        if let Self::Select(node) = self {
            Some(node)
        } else {
            None
        }
    }

    pub fn as_terrace_mut(&mut self) -> Option<&mut TerraceNode> {
        if let Self::Terrace(node) = self {
            Some(node)
        } else {
            None
        }
    }

    pub fn as_unary_mut(&mut self) -> Option<&mut UnaryNode> {
        if let Self::Abs(node) | Self::Negate(node) = self {
            Some(node)
        } else {
            None
        }
    }

    pub fn as_worley_mut(&mut self) -> Option<&mut WorleyNode> {
        if let Self::Worley(node) = self {
            Some(node)
        } else {
            None
        }
    }

    pub fn expr(&self, snarl: &Snarl<Self>) -> Expr {
        match self {
            Self::Abs(node) => Expr::Abs(
                node.input_node_idx
                    .map(|node_idx| Box::new(snarl.get_node(node_idx).expr(snarl)))
                    .unwrap_or_else(|| Box::new(Expr::F64(0.0))),
            ),
            Self::Add(node) => Expr::Add(
                node.input_node_indices
                    .iter()
                    .map(|node_idx| {
                        node_idx
                            .map(|node_idx| Box::new(snarl.get_node(node_idx).expr(snarl)))
                            .unwrap_or_else(|| Box::new(Expr::F64(0.0)))
                    })
                    .collect::<Vec<_>>()
                    .try_into()
                    .unwrap(),
            ),
            Self::BasicMulti(node) => Expr::BasicMulti(FractalExpr {
                source: node.source,
                seed: node.seed.value(snarl),
                octaves: node.octaves.value(snarl),
                frequency: node.frequency.value(snarl),
                lacunarity: node.lacunarity.value(snarl),
                persistence: node.persistence.value(snarl),
            }),
            Self::Billow(node) => Expr::Billow(FractalExpr {
                source: node.source,
                seed: node.seed.value(snarl),
                octaves: node.octaves.value(snarl),
                frequency: node.frequency.value(snarl),
                lacunarity: node.lacunarity.value(snarl),
                persistence: node.persistence.value(snarl),
            }),
            Self::Blend(node) => Expr::Blend(BlendExpr {
                sources: node
                    .input_node_indices
                    .iter()
                    .map(|node_idx| {
                        node_idx
                            .map(|node_idx| Box::new(snarl.get_node(node_idx).expr(snarl)))
                            .unwrap_or_else(|| Box::new(Expr::F64(0.0)))
                    })
                    .collect::<Vec<_>>()
                    .try_into()
                    .unwrap(),
                control: node
                    .control_node_idx
                    .map(|node_idx| Box::new(snarl.get_node(node_idx).expr(snarl)))
                    .unwrap_or_else(|| Box::new(Expr::F64(0.0))),
            }),
            Self::Checkerboard(node) => Expr::Checkerboard(node.size.value(snarl)),
            Self::Clamp(node) => Expr::Clamp(ClampExpr {
                source: node
                    .input_node_idx
                    .map(|node_idx| Box::new(snarl.get_node(node_idx).expr(snarl)))
                    .unwrap_or_else(|| Box::new(Expr::F64(0.0))),
                lower_bound: node.lower_bound.value(snarl),
                upper_bound: node.upper_bound.value(snarl),
            }),
            Self::Curve(node) => Expr::Curve(CurveExpr {
                source: node
                    .input_node_idx
                    .map(|node_idx| Box::new(snarl.get_node(node_idx).expr(snarl)))
                    .unwrap_or_else(|| Box::new(Expr::F64(0.0))),
                control_points: node
                    .control_point_node_indices
                    .iter()
                    .copied()
                    .filter_map(|node_idx| {
                        node_idx.map(|node_idx| {
                            snarl
                                .get_node(node_idx)
                                .as_control_point()
                                .map(|control_point| ControlPointExpr {
                                    input_value: control_point.input.value(snarl),
                                    output_value: control_point.output.value(snarl),
                                })
                                .unwrap()
                        })
                    })
                    .collect(),
            }),
            Self::Cylinders(node) => Expr::Cylinders(node.frequency.value(snarl)),
            Self::Exponent(node) => Expr::Exponent(ExponentExpr {
                source: node
                    .input_node_idx
                    .map(|node_idx| Box::new(snarl.get_node(node_idx).expr(snarl)))
                    .unwrap_or_else(|| Box::new(Expr::F64(0.0))),
                exponent: node.exponent.value(snarl),
            }),
            Self::F64(node) => Expr::F64(node.value),
            Self::Fbm(node) => Expr::Fbm(FractalExpr {
                source: node.source,
                seed: node.seed.value(snarl),
                octaves: node.octaves.value(snarl),
                frequency: node.frequency.value(snarl),
                lacunarity: node.lacunarity.value(snarl),
                persistence: node.persistence.value(snarl),
            }),
            Self::HybridMulti(node) => Expr::HybridMulti(FractalExpr {
                source: node.source,
                seed: node.seed.value(snarl),
                octaves: node.octaves.value(snarl),
                frequency: node.frequency.value(snarl),
                lacunarity: node.lacunarity.value(snarl),
                persistence: node.persistence.value(snarl),
            }),
            Self::Max(node) => Expr::Max(
                node.input_node_indices
                    .iter()
                    .map(|node_idx| {
                        node_idx
                            .map(|node_idx| Box::new(snarl.get_node(node_idx).expr(snarl)))
                            .unwrap_or_else(|| Box::new(Expr::F64(0.0)))
                    })
                    .collect::<Vec<_>>()
                    .try_into()
                    .unwrap(),
            ),
            Self::Min(node) => Expr::Min(
                node.input_node_indices
                    .iter()
                    .map(|node_idx| {
                        node_idx
                            .map(|node_idx| Box::new(snarl.get_node(node_idx).expr(snarl)))
                            .unwrap_or_else(|| Box::new(Expr::F64(0.0)))
                    })
                    .collect::<Vec<_>>()
                    .try_into()
                    .unwrap(),
            ),
            Self::Multiply(node) => Expr::Multiply(
                node.input_node_indices
                    .iter()
                    .map(|node_idx| {
                        node_idx
                            .map(|node_idx| Box::new(snarl.get_node(node_idx).expr(snarl)))
                            .unwrap_or_else(|| Box::new(Expr::F64(0.0)))
                    })
                    .collect::<Vec<_>>()
                    .try_into()
                    .unwrap(),
            ),
            Self::Negate(node) => Expr::Negate(
                node.input_node_idx
                    .map(|node_idx| Box::new(snarl.get_node(node_idx).expr(snarl)))
                    .unwrap_or_else(|| Box::new(Expr::F64(0.0))),
            ),
            Self::OpenSimplex(node) => Expr::OpenSimplex(node.seed.value(snarl)),
            Self::Perlin(node) => Expr::Perlin(node.seed.value(snarl)),
            Self::PerlinSurflet(node) => Expr::PerlinSurflet(node.seed.value(snarl)),
            Self::Power(node) => Expr::Power(
                node.input_node_indices
                    .iter()
                    .map(|node_idx| {
                        node_idx
                            .map(|node_idx| Box::new(snarl.get_node(node_idx).expr(snarl)))
                            .unwrap_or_else(|| Box::new(Expr::F64(0.0)))
                    })
                    .collect::<Vec<_>>()
                    .try_into()
                    .unwrap(),
            ),
            Self::RigidMulti(node) => Expr::RidgedMulti(RigidFractalExpr {
                source: node.source,
                seed: node.seed.value(snarl),
                octaves: node.octaves.value(snarl),
                frequency: node.frequency.value(snarl),
                lacunarity: node.lacunarity.value(snarl),
                persistence: node.persistence.value(snarl),
                attenuation: node.attenuation.value(snarl),
            }),
            Self::ScaleBias(node) => Expr::ScaleBias(ScaleBiasExpr {
                source: node
                    .input_node_idx
                    .map(|node_idx| Box::new(snarl.get_node(node_idx).expr(snarl)))
                    .unwrap_or_else(|| Box::new(Expr::F64(0.0))),
                scale: node.scale.value(snarl),
                bias: node.bias.value(snarl),
            }),
            Self::Select(node) => Expr::Select(SelectExpr {
                sources: node
                    .input_node_indices
                    .iter()
                    .map(|node_idx| {
                        node_idx
                            .map(|node_idx| Box::new(snarl.get_node(node_idx).expr(snarl)))
                            .unwrap_or_else(|| Box::new(Expr::F64(0.0)))
                    })
                    .collect::<Vec<_>>()
                    .try_into()
                    .unwrap(),
                control: node
                    .control_node_idx
                    .map(|node_idx| Box::new(snarl.get_node(node_idx).expr(snarl)))
                    .unwrap_or_else(|| Box::new(Expr::F64(0.0))),
                lower_bound: node.lower_bound.value(snarl),
                upper_bound: node.upper_bound.value(snarl),
                falloff: node.falloff.value(snarl),
            }),
            Self::Simplex(node) => Expr::Simplex(node.seed.value(snarl)),
            Self::SuperSimplex(node) => Expr::SuperSimplex(node.seed.value(snarl)),
            Self::Terrace(node) => Expr::Terrace(TerraceExpr {
                source: node
                    .input_node_idx
                    .map(|node_idx| Box::new(snarl.get_node(node_idx).expr(snarl)))
                    .unwrap_or_else(|| Box::new(Expr::F64(0.0))),
                inverted: node.inverted,
                control_points: node
                    .control_point_node_indices
                    .iter()
                    .copied()
                    .filter_map(|node_idx| {
                        node_idx.map(|node_idx| snarl.get_node(node_idx).as_const_f64().unwrap())
                    })
                    .collect(),
            }),
            Self::Value(node) => Expr::Value(node.seed.value(snarl)),
            Self::Worley(node) => Expr::Worley(WorleyExpr {
                seed: node.seed.value(snarl),
                frequency: node.frequency.value(snarl),
                distance_fn: node.distance_fn,
                return_ty: node.return_ty,
            }),
            Self::ControlPoint(_) | Self::U32(_) => unreachable!(),
        }
    }

    pub fn has_image(&self) -> bool {
        self.image().is_some()
    }

    pub fn image(&self) -> Option<&Image> {
        match self {
            Self::Abs(UnaryNode { image, .. })
            | Self::Add(CombinerNode { image, .. })
            | Self::BasicMulti(FractalNode { image, .. })
            | Self::Billow(FractalNode { image, .. })
            | Self::Blend(BlendNode { image, .. })
            | Self::Checkerboard(CheckerboardNode { image, .. })
            | Self::Clamp(ClampNode { image, .. })
            | Self::Curve(CurveNode { image, .. })
            | Self::Cylinders(CylindersNode { image, .. })
            | Self::Exponent(ExponentNode { image, .. })
            | Self::Fbm(FractalNode { image, .. })
            | Self::HybridMulti(FractalNode { image, .. })
            | Self::Max(CombinerNode { image, .. })
            | Self::Min(CombinerNode { image, .. })
            | Self::Multiply(CombinerNode { image, .. })
            | Self::Negate(UnaryNode { image, .. })
            | Self::OpenSimplex(GeneratorNode { image, .. })
            | Self::Perlin(GeneratorNode { image, .. })
            | Self::PerlinSurflet(GeneratorNode { image, .. })
            | Self::Power(CombinerNode { image, .. })
            | Self::RigidMulti(RigidFractalNode { image, .. })
            | Self::ScaleBias(ScaleBiasNode { image, .. })
            | Self::Select(SelectNode { image, .. })
            | Self::Simplex(GeneratorNode { image, .. })
            | Self::SuperSimplex(GeneratorNode { image, .. })
            | Self::Terrace(TerraceNode { image, .. })
            | Self::Value(GeneratorNode { image, .. })
            | Self::Worley(WorleyNode { image, .. }) => Some(image),
            Self::ControlPoint(_) | Self::F64(_) | Self::U32(_) => None,
        }
    }

    pub fn image_mut(&mut self) -> Option<&mut Image> {
        match self {
            Self::Abs(UnaryNode { image, .. })
            | Self::Add(CombinerNode { image, .. })
            | Self::BasicMulti(FractalNode { image, .. })
            | Self::Billow(FractalNode { image, .. })
            | Self::Blend(BlendNode { image, .. })
            | Self::Checkerboard(CheckerboardNode { image, .. })
            | Self::Clamp(ClampNode { image, .. })
            | Self::Curve(CurveNode { image, .. })
            | Self::Cylinders(CylindersNode { image, .. })
            | Self::Exponent(ExponentNode { image, .. })
            | Self::Fbm(FractalNode { image, .. })
            | Self::HybridMulti(FractalNode { image, .. })
            | Self::Max(CombinerNode { image, .. })
            | Self::Min(CombinerNode { image, .. })
            | Self::Multiply(CombinerNode { image, .. })
            | Self::Negate(UnaryNode { image, .. })
            | Self::OpenSimplex(GeneratorNode { image, .. })
            | Self::Perlin(GeneratorNode { image, .. })
            | Self::PerlinSurflet(GeneratorNode { image, .. })
            | Self::Power(CombinerNode { image, .. })
            | Self::RigidMulti(RigidFractalNode { image, .. })
            | Self::ScaleBias(ScaleBiasNode { image, .. })
            | Self::Select(SelectNode { image, .. })
            | Self::Simplex(GeneratorNode { image, .. })
            | Self::SuperSimplex(GeneratorNode { image, .. })
            | Self::Terrace(TerraceNode { image, .. })
            | Self::Value(GeneratorNode { image, .. })
            | Self::Worley(WorleyNode { image, .. }) => Some(image),
            Self::ControlPoint(_) | Self::F64(_) | Self::U32(_) => None,
        }
    }

    pub fn output_node_indices(&self) -> &HashSet<usize> {
        match self {
            Self::Abs(UnaryNode {
                output_node_indices,
                ..
            })
            | Self::Add(CombinerNode {
                output_node_indices,
                ..
            })
            | Self::BasicMulti(FractalNode {
                output_node_indices,
                ..
            })
            | Self::Billow(FractalNode {
                output_node_indices,
                ..
            })
            | Self::Blend(BlendNode {
                output_node_indices,
                ..
            })
            | Self::Checkerboard(CheckerboardNode {
                output_node_indices,
                ..
            })
            | Self::Clamp(ClampNode {
                output_node_indices,
                ..
            })
            | Self::ControlPoint(ControlPointNode {
                output_node_indices,
                ..
            })
            | Self::Curve(CurveNode {
                output_node_indices,
                ..
            })
            | Self::Cylinders(CylindersNode {
                output_node_indices,
                ..
            })
            | Self::Exponent(ExponentNode {
                output_node_indices,
                ..
            })
            | Self::F64(ConstantNode {
                output_node_indices,
                ..
            })
            | Self::Fbm(FractalNode {
                output_node_indices,
                ..
            })
            | Self::HybridMulti(FractalNode {
                output_node_indices,
                ..
            })
            | Self::Max(CombinerNode {
                output_node_indices,
                ..
            })
            | Self::Min(CombinerNode {
                output_node_indices,
                ..
            })
            | Self::Multiply(CombinerNode {
                output_node_indices,
                ..
            })
            | Self::Negate(UnaryNode {
                output_node_indices,
                ..
            })
            | Self::OpenSimplex(GeneratorNode {
                output_node_indices,
                ..
            })
            | Self::Perlin(GeneratorNode {
                output_node_indices,
                ..
            })
            | Self::PerlinSurflet(GeneratorNode {
                output_node_indices,
                ..
            })
            | Self::Power(CombinerNode {
                output_node_indices,
                ..
            })
            | Self::RigidMulti(RigidFractalNode {
                output_node_indices,
                ..
            })
            | Self::ScaleBias(ScaleBiasNode {
                output_node_indices,
                ..
            })
            | Self::Select(SelectNode {
                output_node_indices,
                ..
            })
            | Self::Simplex(GeneratorNode {
                output_node_indices,
                ..
            })
            | Self::SuperSimplex(GeneratorNode {
                output_node_indices,
                ..
            })
            | Self::Terrace(TerraceNode {
                output_node_indices,
                ..
            })
            | Self::U32(ConstantNode {
                output_node_indices,
                ..
            })
            | Self::Value(GeneratorNode {
                output_node_indices,
                ..
            })
            | Self::Worley(WorleyNode {
                output_node_indices,
                ..
            }) => output_node_indices,
        }
    }

    pub fn output_node_indices_mut(&mut self) -> &mut HashSet<usize> {
        match self {
            Self::Abs(UnaryNode {
                output_node_indices,
                ..
            })
            | Self::Add(CombinerNode {
                output_node_indices,
                ..
            })
            | Self::BasicMulti(FractalNode {
                output_node_indices,
                ..
            })
            | Self::Billow(FractalNode {
                output_node_indices,
                ..
            })
            | Self::Blend(BlendNode {
                output_node_indices,
                ..
            })
            | Self::Checkerboard(CheckerboardNode {
                output_node_indices,
                ..
            })
            | Self::Clamp(ClampNode {
                output_node_indices,
                ..
            })
            | Self::ControlPoint(ControlPointNode {
                output_node_indices,
                ..
            })
            | Self::Curve(CurveNode {
                output_node_indices,
                ..
            })
            | Self::Cylinders(CylindersNode {
                output_node_indices,
                ..
            })
            | Self::Exponent(ExponentNode {
                output_node_indices,
                ..
            })
            | Self::F64(ConstantNode {
                output_node_indices,
                ..
            })
            | Self::Fbm(FractalNode {
                output_node_indices,
                ..
            })
            | Self::HybridMulti(FractalNode {
                output_node_indices,
                ..
            })
            | Self::Max(CombinerNode {
                output_node_indices,
                ..
            })
            | Self::Min(CombinerNode {
                output_node_indices,
                ..
            })
            | Self::Multiply(CombinerNode {
                output_node_indices,
                ..
            })
            | Self::Negate(UnaryNode {
                output_node_indices,
                ..
            })
            | Self::OpenSimplex(GeneratorNode {
                output_node_indices,
                ..
            })
            | Self::Perlin(GeneratorNode {
                output_node_indices,
                ..
            })
            | Self::PerlinSurflet(GeneratorNode {
                output_node_indices,
                ..
            })
            | Self::Power(CombinerNode {
                output_node_indices,
                ..
            })
            | Self::RigidMulti(RigidFractalNode {
                output_node_indices,
                ..
            })
            | Self::ScaleBias(ScaleBiasNode {
                output_node_indices,
                ..
            })
            | Self::Select(SelectNode {
                output_node_indices,
                ..
            })
            | Self::Simplex(GeneratorNode {
                output_node_indices,
                ..
            })
            | Self::SuperSimplex(GeneratorNode {
                output_node_indices,
                ..
            })
            | Self::Terrace(TerraceNode {
                output_node_indices,
                ..
            })
            | Self::U32(ConstantNode {
                output_node_indices,
                ..
            })
            | Self::Value(GeneratorNode {
                output_node_indices,
                ..
            })
            | Self::Worley(WorleyNode {
                output_node_indices,
                ..
            }) => output_node_indices,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum ReturnType {
    Distance,
    Value,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct RigidFractalNode {
    pub image: Image,

    pub output_node_indices: HashSet<usize>,

    pub source: Source,
    pub seed: NodeValue<u32>,
    pub octaves: NodeValue<u32>,
    pub frequency: NodeValue<f64>,
    pub lacunarity: NodeValue<f64>,
    pub persistence: NodeValue<f64>,
    pub attenuation: NodeValue<f64>,
}

impl Default for RigidFractalNode {
    fn default() -> Self {
        Self {
            image: Default::default(),
            output_node_indices: Default::default(),
            source: Default::default(),
            seed: NodeValue::Value(RigidFractal::<AnySeedable>::DEFAULT_SEED),
            octaves: NodeValue::Value(RigidFractal::<AnySeedable>::DEFAULT_OCTAVE_COUNT as _),
            frequency: NodeValue::Value(RigidFractal::<AnySeedable>::DEFAULT_FREQUENCY),
            lacunarity: NodeValue::Value(RigidFractal::<AnySeedable>::DEFAULT_LACUNARITY),
            persistence: NodeValue::Value(RigidFractal::<AnySeedable>::DEFAULT_PERSISTENCE),
            attenuation: NodeValue::Value(RigidFractal::<AnySeedable>::DEFAULT_ATTENUATION),
        }
    }
}

#[derive(Clone, Default, Serialize, Deserialize)]
pub struct ScaleBiasNode {
    pub image: Image,

    pub input_node_idx: Option<usize>,
    pub output_node_indices: HashSet<usize>,

    pub scale: NodeValue<f64>,
    pub bias: NodeValue<f64>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct SelectNode {
    pub image: Image,

    pub input_node_indices: [Option<usize>; 2],
    pub control_node_idx: Option<usize>,
    pub output_node_indices: HashSet<usize>,

    pub lower_bound: NodeValue<f64>,
    pub upper_bound: NodeValue<f64>,
    pub falloff: NodeValue<f64>,
}

impl Default for SelectNode {
    fn default() -> Self {
        Self {
            image: Default::default(),
            input_node_indices: Default::default(),
            control_node_idx: Default::default(),
            output_node_indices: Default::default(),
            lower_bound: NodeValue::Value(0.0),
            upper_bound: NodeValue::Value(1.0),
            falloff: NodeValue::Value(0.0),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum Source {
    OpenSimplex,
    Perlin,
    PerlinSurflet,
    Simplex,
    SuperSimplex,
    Value,
    Worley,
}

impl Default for Source {
    fn default() -> Self {
        Self::Perlin
    }
}

#[derive(Clone, Default, Serialize, Deserialize)]
pub struct TerraceNode {
    pub image: Image,

    pub input_node_idx: Option<usize>,
    pub output_node_indices: HashSet<usize>,

    pub inverted: bool,
    pub control_point_node_indices: Vec<Option<usize>>,
}

#[derive(Clone, Default, Serialize, Deserialize)]
pub struct UnaryNode {
    pub image: Image,

    pub input_node_idx: Option<usize>,
    pub output_node_indices: HashSet<usize>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct WorleyNode {
    pub image: Image,

    pub output_node_indices: HashSet<usize>,

    pub seed: NodeValue<u32>,
    pub frequency: NodeValue<f64>,
    pub distance_fn: DistanceFunction,
    pub return_ty: ReturnType,
}

impl Default for WorleyNode {
    fn default() -> Self {
        Self {
            image: Default::default(),
            output_node_indices: Default::default(),
            seed: NodeValue::Value(Worley::DEFAULT_SEED),
            frequency: NodeValue::Value(Worley::DEFAULT_FREQUENCY),
            distance_fn: DistanceFunction::Euclidean,
            return_ty: ReturnType::Value,
        }
    }
}
