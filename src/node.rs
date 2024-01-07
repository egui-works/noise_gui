use {
    super::expr::{
        BlendExpr, ClampExpr, ControlPointExpr, CurveExpr, DisplaceExpr, DistanceFunction,
        ExponentExpr, Expr, FractalExpr, OpType, ReturnType, RigidFractalExpr, ScaleBiasExpr,
        SelectExpr, SourceType, TerraceExpr, TransformExpr, TurbulenceExpr, Variable, WorleyExpr,
    },
    egui::TextureHandle,
    egui_snarl::{OutPinId, Snarl},
    noise::{
        BasicMulti as Fractal, Cylinders, Perlin as AnySeedable, RidgedMulti as RigidFractal,
        Turbulence, Worley,
    },
    serde::{Deserialize, Serialize},
    std::{cell::RefCell, collections::HashSet},
};

fn constant(value: f64) -> Box<Expr> {
    Box::new(Expr::Constant(Variable::Anonymous(value)))
}

#[derive(Clone, Default, Serialize, Deserialize)]
pub struct BlendNode {
    pub image: Image,

    pub input_node_indices: [Option<usize>; 2],
    pub control_node_idx: Option<usize>,
}

impl BlendNode {
    fn expr(&self, snarl: &Snarl<NoiseNode>) -> BlendExpr {
        BlendExpr {
            sources: self
                .input_node_indices
                .iter()
                .map(|node_idx| {
                    node_idx
                        .map(|node_idx| Box::new(snarl.get_node(node_idx).expr(snarl)))
                        .unwrap_or_else(|| constant(0.0))
                })
                .collect::<Vec<_>>()
                .try_into()
                .unwrap(),
            control: self
                .control_node_idx
                .map(|node_idx| Box::new(snarl.get_node(node_idx).expr(snarl)))
                .unwrap_or_else(|| constant(0.0)),
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct CheckerboardNode {
    pub image: Image,

    pub size: NodeValue<u32>,
}

impl Default for CheckerboardNode {
    fn default() -> Self {
        Self {
            image: Default::default(),
            size: NodeValue::Value(0), // TODO: Checkerboard::DEFAULT_SIZE is private!
        }
    }
}

#[derive(Clone, Default, Serialize, Deserialize)]
pub struct ClampNode {
    pub image: Image,

    pub input_node_idx: Option<usize>,

    pub lower_bound: NodeValue<f64>,
    pub upper_bound: NodeValue<f64>,
}

impl ClampNode {
    fn expr(&self, snarl: &Snarl<NoiseNode>) -> ClampExpr {
        ClampExpr {
            source: self
                .input_node_idx
                .map(|node_idx| Box::new(snarl.get_node(node_idx).expr(snarl)))
                .unwrap_or_else(|| constant(0.0)),
            lower_bound: self.lower_bound.var(snarl),
            upper_bound: self.upper_bound.var(snarl),
        }
    }
}

#[derive(Clone, Default, Serialize, Deserialize)]
pub struct CombinerNode {
    pub image: Image,

    pub input_node_indices: [Option<usize>; 2],
}

impl CombinerNode {
    fn expr(&self, snarl: &Snarl<NoiseNode>, default_value: f64) -> [Box<Expr>; 2] {
        self.input_node_indices
            .iter()
            .map(|node_idx| {
                node_idx
                    .map(|node_idx| Box::new(snarl.get_node(node_idx).expr(snarl)))
                    .unwrap_or_else(|| constant(default_value))
            })
            .collect::<Vec<_>>()
            .try_into()
            .unwrap()
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct ConstantNode<T> {
    pub name: String,

    pub value: T,
}

impl<T> Default for ConstantNode<T>
where
    T: Default,
{
    fn default() -> Self {
        Self {
            name: "name".to_owned(),
            value: Default::default(),
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct ConstantOpNode<T> {
    pub inputs: [NodeValue<T>; 2],

    pub op_ty: OpType,
}

impl<T> ConstantOpNode<T> {
    pub fn new(op_ty: OpType, value: T) -> Self
    where
        T: Copy,
    {
        Self {
            inputs: [NodeValue::Value(value); 2],
            op_ty,
        }
    }
}

impl ConstantOpNode<f64> {
    fn var(&self, snarl: &Snarl<NoiseNode>) -> Variable<f64> {
        Variable::Operation(
            self.inputs
                .iter()
                .map(|input| Box::new(input.var(snarl)))
                .collect::<Vec<_>>()
                .try_into()
                .unwrap(),
            self.op_ty,
        )
    }
}

#[derive(Clone, Default, Serialize, Deserialize)]
pub struct ControlPointNode {
    pub input: NodeValue<f64>,
    pub output: NodeValue<f64>,
}

#[derive(Clone, Default, Serialize, Deserialize)]
pub struct CurveNode {
    pub image: Image,

    pub input_node_idx: Option<usize>,

    pub control_point_node_indices: Vec<Option<usize>>,
}

impl CurveNode {
    fn expr(&self, snarl: &Snarl<NoiseNode>) -> CurveExpr {
        CurveExpr {
            source: self
                .input_node_idx
                .map(|node_idx| Box::new(snarl.get_node(node_idx).expr(snarl)))
                .unwrap_or_else(|| constant(0.0)),
            control_points: self
                .control_point_node_indices
                .iter()
                .copied()
                .filter_map(|node_idx| {
                    node_idx.map(|node_idx| {
                        snarl
                            .get_node(node_idx)
                            .as_control_point()
                            .map(|control_point| ControlPointExpr {
                                input_value: control_point.input.var(snarl),
                                output_value: control_point.output.var(snarl),
                            })
                            .unwrap()
                    })
                })
                .collect(),
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct CylindersNode {
    pub image: Image,

    pub frequency: NodeValue<f64>,
}

impl Default for CylindersNode {
    fn default() -> Self {
        Self {
            image: Default::default(),
            frequency: NodeValue::Value(Cylinders::DEFAULT_FREQUENCY),
        }
    }
}

#[derive(Clone, Default, Serialize, Deserialize)]
pub struct DisplaceNode {
    pub image: Image,

    pub input_node_idx: Option<usize>,

    pub axes: [Option<usize>; 4],
}

impl DisplaceNode {
    fn expr(&self, snarl: &Snarl<NoiseNode>) -> DisplaceExpr {
        DisplaceExpr {
            source: self
                .input_node_idx
                .map(|node_idx| Box::new(snarl.get_node(node_idx).expr(snarl)))
                .unwrap_or_else(|| constant(0.0)),
            axes: self
                .axes
                .iter()
                .map(|axis| {
                    axis.map(|node_idx| Box::new(snarl.get_node(node_idx).expr(snarl)))
                        .unwrap_or_else(|| constant(0.0))
                })
                .collect::<Vec<_>>()
                .try_into()
                .unwrap(),
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct ExponentNode {
    pub image: Image,

    pub input_node_idx: Option<usize>,

    pub exponent: NodeValue<f64>,
}

impl ExponentNode {
    fn expr(&self, snarl: &Snarl<NoiseNode>) -> ExponentExpr {
        ExponentExpr {
            source: self
                .input_node_idx
                .map(|node_idx| Box::new(snarl.get_node(node_idx).expr(snarl)))
                .unwrap_or_else(|| constant(0.0)),
            exponent: self.exponent.var(snarl),
        }
    }
}

impl Default for ExponentNode {
    fn default() -> Self {
        Self {
            image: Default::default(),
            input_node_idx: Default::default(),
            exponent: NodeValue::Value(1.0),
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct FractalNode {
    pub image: Image,

    pub source_ty: SourceType,
    pub seed: NodeValue<u32>,
    pub octaves: NodeValue<u32>,
    pub frequency: NodeValue<f64>,
    pub lacunarity: NodeValue<f64>,
    pub persistence: NodeValue<f64>,
}

impl FractalNode {
    fn expr(&self, snarl: &Snarl<NoiseNode>) -> FractalExpr {
        FractalExpr {
            source_ty: self.source_ty,
            seed: self.seed.var(snarl),
            octaves: self.octaves.var(snarl),
            frequency: self.frequency.var(snarl),
            lacunarity: self.lacunarity.var(snarl),
            persistence: self.persistence.var(snarl),
        }
    }
}

impl Default for FractalNode {
    fn default() -> Self {
        Self {
            image: Default::default(),
            source_ty: Default::default(),
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

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
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

    pub fn as_value_mut(&mut self) -> Option<&mut T> {
        if let Self::Value(value) = self {
            Some(value)
        } else {
            None
        }
    }

    pub fn is_node_idx(&self) -> bool {
        self.as_node_index().is_some()
    }
}

impl NodeValue<f64> {
    fn eval(self, snarl: &Snarl<NoiseNode>) -> f64 {
        match self {
            Self::Node(node_idx) => snarl.get_node(node_idx).eval_f64(snarl),
            Self::Value(value) => value,
        }
    }

    fn var(self, snarl: &Snarl<NoiseNode>) -> Variable<f64> {
        match self {
            Self::Node(node_idx) => match snarl.get_node(node_idx) {
                NoiseNode::F64(node) => Variable::Named(node.name.clone(), node.value),
                NoiseNode::F64Operation(node) => node.var(snarl),
                _ => unreachable!(),
            },
            Self::Value(value) => Variable::Anonymous(value),
        }
    }
}

impl NodeValue<u32> {
    fn eval(self, snarl: &Snarl<NoiseNode>) -> u32 {
        match self {
            Self::Node(node_idx) => snarl.get_node(node_idx).eval_u32(snarl),
            Self::Value(value) => value,
        }
    }

    fn var(self, snarl: &Snarl<NoiseNode>) -> Variable<u32> {
        match self {
            Self::Node(node_idx) => match snarl.get_node(node_idx) {
                NoiseNode::U32(node) => Variable::Named(node.name.clone(), node.value),
                NoiseNode::U32Operation(node) => Variable::Operation(
                    node.inputs
                        .iter()
                        .map(|input| Box::new(input.var(snarl)))
                        .collect::<Vec<_>>()
                        .try_into()
                        .unwrap(),
                    node.op_ty,
                ),
                _ => unreachable!(),
            },
            Self::Value(value) => Variable::Anonymous(value),
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
    Displace(DisplaceNode),
    Exponent(ExponentNode),
    F64(ConstantNode<f64>),
    F64Operation(ConstantOpNode<f64>),
    Fbm(FractalNode),
    HybridMulti(FractalNode),
    Max(CombinerNode),
    Min(CombinerNode),
    Multiply(CombinerNode),
    Negate(UnaryNode),
    OpenSimplex(GeneratorNode),
    Operation(ConstantOpNode<()>),
    Perlin(GeneratorNode),
    PerlinSurflet(GeneratorNode),
    Power(CombinerNode),
    RigidMulti(RigidFractalNode),
    RotatePoint(TransformNode),
    ScaleBias(ScaleBiasNode),
    ScalePoint(TransformNode),
    Select(SelectNode),
    Simplex(GeneratorNode),
    SuperSimplex(GeneratorNode),
    Terrace(TerraceNode),
    TranslatePoint(TransformNode),
    Turbulence(TurbulenceNode),
    U32(ConstantNode<u32>),
    U32Operation(ConstantOpNode<u32>),
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

    pub fn as_const_op_f64(&self) -> Option<&ConstantOpNode<f64>> {
        if let Self::F64Operation(node) = self {
            Some(node)
        } else {
            None
        }
    }

    pub fn as_const_op_f64_mut(&mut self) -> Option<&mut ConstantOpNode<f64>> {
        if let Self::F64Operation(node) = self {
            Some(node)
        } else {
            None
        }
    }

    pub fn as_const_op_tuple(&self) -> Option<&ConstantOpNode<()>> {
        if let Self::Operation(node) = self {
            Some(node)
        } else {
            None
        }
    }

    pub fn as_const_op_tuple_mut(&mut self) -> Option<&mut ConstantOpNode<()>> {
        if let Self::Operation(node) = self {
            Some(node)
        } else {
            None
        }
    }

    pub fn as_const_op_u32(&self) -> Option<&ConstantOpNode<u32>> {
        if let Self::U32Operation(node) = self {
            Some(node)
        } else {
            None
        }
    }

    pub fn as_const_op_u32_mut(&mut self) -> Option<&mut ConstantOpNode<u32>> {
        if let Self::U32Operation(node) = self {
            Some(node)
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

    pub fn as_displace_mut(&mut self) -> Option<&mut DisplaceNode> {
        if let Self::Displace(node) = self {
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

    pub fn as_transform_mut(&mut self) -> Option<&mut TransformNode> {
        if let Self::RotatePoint(node) | Self::ScalePoint(node) | Self::TranslatePoint(node) = self
        {
            Some(node)
        } else {
            None
        }
    }

    pub fn as_turbulence_mut(&mut self) -> Option<&mut TurbulenceNode> {
        if let Self::Turbulence(node) = self {
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

    pub fn eval_f64(&self, snarl: &Snarl<Self>) -> f64 {
        match self {
            Self::F64(node) => node.value,
            Self::F64Operation(node) => {
                let (lhs, rhs) = (node.inputs[0].eval(snarl), node.inputs[1].eval(snarl));
                match node.op_ty {
                    OpType::Add => lhs + rhs,
                    OpType::Divide => {
                        if rhs != 0.0 {
                            lhs / rhs
                        } else {
                            0.0
                        }
                    }
                    OpType::Multiply => lhs * rhs,
                    OpType::Subtract => lhs - rhs,
                }
            }
            _ => unreachable!(),
        }
    }

    pub fn eval_u32(&self, snarl: &Snarl<Self>) -> u32 {
        match self {
            Self::U32(node) => node.value,
            Self::U32Operation(node) => {
                let (lhs, rhs) = (node.inputs[0].eval(snarl), node.inputs[1].eval(snarl));
                match node.op_ty {
                    OpType::Add => lhs.checked_add(rhs),
                    OpType::Divide => lhs.checked_div(rhs),
                    OpType::Multiply => lhs.checked_mul(rhs),
                    OpType::Subtract => lhs.checked_sub(rhs),
                }
                .unwrap_or_default()
            }
            _ => unreachable!(),
        }
    }

    pub fn expr(&self, snarl: &Snarl<Self>) -> Expr {
        match self {
            Self::Abs(node) => Expr::Abs(node.expr(snarl)),
            Self::Add(node) => Expr::Add(node.expr(snarl, 0.0)),
            Self::BasicMulti(node) => Expr::BasicMulti(node.expr(snarl)),
            Self::Billow(node) => Expr::Billow(node.expr(snarl)),
            Self::Blend(node) => Expr::Blend(node.expr(snarl)),
            Self::Checkerboard(node) => Expr::Checkerboard(node.size.var(snarl)),
            Self::Clamp(node) => Expr::Clamp(node.expr(snarl)),
            Self::Curve(node) => Expr::Curve(node.expr(snarl)),
            Self::Cylinders(node) => Expr::Cylinders(node.frequency.var(snarl)),
            Self::Displace(node) => Expr::Displace(node.expr(snarl)),
            Self::Exponent(node) => Expr::Exponent(node.expr(snarl)),
            Self::F64(node) => Expr::Constant(Variable::Named(node.name.clone(), node.value)),
            Self::F64Operation(node) => Expr::Constant(node.var(snarl)),
            Self::Fbm(node) => Expr::Fbm(node.expr(snarl)),
            Self::HybridMulti(node) => Expr::HybridMulti(node.expr(snarl)),
            Self::Max(node) => Expr::Max(node.expr(snarl, 1.0)),
            Self::Min(node) => Expr::Min(node.expr(snarl, -1.0)),
            Self::Multiply(node) => Expr::Multiply(node.expr(snarl, 1.0)),
            Self::Negate(node) => Expr::Negate(node.expr(snarl)),
            Self::OpenSimplex(node) => Expr::OpenSimplex(node.seed.var(snarl)),
            Self::Perlin(node) => Expr::Perlin(node.seed.var(snarl)),
            Self::PerlinSurflet(node) => Expr::PerlinSurflet(node.seed.var(snarl)),
            Self::Power(node) => Expr::Power(node.expr(snarl, 1.0)),
            Self::RigidMulti(node) => Expr::RidgedMulti(node.expr(snarl)),
            Self::RotatePoint(node) => Expr::RotatePoint(node.expr(snarl)),
            Self::ScaleBias(node) => Expr::ScaleBias(node.expr(snarl)),
            Self::ScalePoint(node) => Expr::ScalePoint(node.expr(snarl)),
            Self::Select(node) => Expr::Select(node.expr(snarl)),
            Self::Simplex(node) => Expr::Simplex(node.seed.var(snarl)),
            Self::SuperSimplex(node) => Expr::SuperSimplex(node.seed.var(snarl)),
            Self::Terrace(node) => Expr::Terrace(node.expr(snarl)),
            Self::TranslatePoint(node) => Expr::TranslatePoint(node.expr(snarl)),
            Self::Turbulence(node) => Expr::Turbulence(node.expr(snarl)),
            Self::Value(node) => Expr::Value(node.seed.var(snarl)),
            Self::Worley(node) => Expr::Worley(node.expr(snarl)),
            Self::ControlPoint(_) | Self::Operation(_) | Self::U32(_) | Self::U32Operation(_) => {
                unreachable!()
            }
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
            | Self::Displace(DisplaceNode { image, .. })
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
            | Self::RotatePoint(TransformNode { image, .. })
            | Self::ScaleBias(ScaleBiasNode { image, .. })
            | Self::ScalePoint(TransformNode { image, .. })
            | Self::Select(SelectNode { image, .. })
            | Self::Simplex(GeneratorNode { image, .. })
            | Self::SuperSimplex(GeneratorNode { image, .. })
            | Self::Terrace(TerraceNode { image, .. })
            | Self::TranslatePoint(TransformNode { image, .. })
            | Self::Turbulence(TurbulenceNode { image, .. })
            | Self::Value(GeneratorNode { image, .. })
            | Self::Worley(WorleyNode { image, .. }) => Some(image),
            Self::ControlPoint(_)
            | Self::F64(_)
            | Self::F64Operation(_)
            | Self::Operation(_)
            | Self::U32(_)
            | Self::U32Operation(_) => None,
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
            | Self::Displace(DisplaceNode { image, .. })
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
            | Self::RotatePoint(TransformNode { image, .. })
            | Self::ScaleBias(ScaleBiasNode { image, .. })
            | Self::ScalePoint(TransformNode { image, .. })
            | Self::Select(SelectNode { image, .. })
            | Self::Simplex(GeneratorNode { image, .. })
            | Self::SuperSimplex(GeneratorNode { image, .. })
            | Self::Terrace(TerraceNode { image, .. })
            | Self::TranslatePoint(TransformNode { image, .. })
            | Self::Turbulence(TurbulenceNode { image, .. })
            | Self::Value(GeneratorNode { image, .. })
            | Self::Worley(WorleyNode { image, .. }) => Some(image),
            Self::ControlPoint(_)
            | Self::F64(_)
            | Self::F64Operation(_)
            | Self::Operation(_)
            | Self::U32(_)
            | Self::U32Operation(_) => None,
        }
    }

    pub fn propagate_f64_from_tuple_op(node_idx: usize, snarl: &mut Snarl<Self>) {
        thread_local! {
            static CHILD_NODE_INDICES: RefCell<Option<HashSet<usize>>> = RefCell::new(Some(Default::default()));
            static NODE_INDICES: RefCell<Option<Vec<usize>>> = RefCell::new(Some(Default::default()));
        }

        let mut child_node_indices = CHILD_NODE_INDICES.take().unwrap();
        let mut node_indices = NODE_INDICES.take().unwrap();
        node_indices.push(node_idx);

        while let Some(node_idx) = node_indices.pop() {
            if child_node_indices.insert(node_idx) {
                node_indices.extend(
                    snarl
                        .out_pin(OutPinId {
                            node: node_idx,
                            output: 0,
                        })
                        .remotes
                        .iter()
                        .map(|remote| remote.node),
                );

                if let node @ Self::Operation(_) = snarl.get_node_mut(node_idx) {
                    let op = node.as_const_op_tuple().unwrap().clone();
                    node_indices.extend(op.inputs.iter().filter_map(|input| input.as_node_index()));

                    *node = NoiseNode::F64Operation(ConstantOpNode {
                        inputs: op
                            .inputs
                            .iter()
                            .copied()
                            .map(|input| {
                                input
                                    .as_node_index()
                                    .map(NodeValue::Node)
                                    .unwrap_or_default()
                            })
                            .collect::<Vec<_>>()
                            .try_into()
                            .unwrap(),
                        op_ty: op.op_ty,
                    });
                } else {
                    unreachable!();
                }
            }
        }

        child_node_indices.clear();
        CHILD_NODE_INDICES.set(Some(child_node_indices));
        NODE_INDICES.set(Some(node_indices));
    }

    pub fn propagate_tuple_from_f64_op(node_idx: usize, snarl: &mut Snarl<Self>) {
        thread_local! {
            static CHILD_NODE_INDICES: RefCell<Option<HashSet<usize>>> = RefCell::new(Some(Default::default()));
            static NODE_INDICES: RefCell<Option<Vec<usize>>> = RefCell::new(Some(Default::default()));
        }

        let mut child_node_indices = CHILD_NODE_INDICES.take().unwrap();
        let mut node_indices = NODE_INDICES.take().unwrap();
        node_indices.push(node_idx);

        while let Some(node_idx) = node_indices.pop() {
            if child_node_indices.insert(node_idx) {
                if let node @ Self::F64Operation(_) = snarl.get_node(node_idx) {
                    let op = node.as_const_op_f64().unwrap();
                    node_indices.extend(op.inputs.iter().filter_map(|input| input.as_node_index()));
                    node_indices.extend(
                        snarl
                            .out_pin(OutPinId {
                                node: node_idx,
                                output: 0,
                            })
                            .remotes
                            .iter()
                            .map(|remote| remote.node),
                    );
                } else {
                    child_node_indices.clear();
                    CHILD_NODE_INDICES.set(Some(child_node_indices));

                    node_indices.clear();
                    NODE_INDICES.set(Some(node_indices));

                    return;
                }
            }
        }

        for node_idx in child_node_indices.drain() {
            let node = snarl.get_node_mut(node_idx);
            let op = node.as_const_op_f64().unwrap().clone();

            *node = NoiseNode::Operation(ConstantOpNode {
                inputs: op
                    .inputs
                    .iter()
                    .copied()
                    .map(|input| {
                        input
                            .as_node_index()
                            .map(NodeValue::Node)
                            .unwrap_or_default()
                    })
                    .collect::<Vec<_>>()
                    .try_into()
                    .unwrap(),
                op_ty: op.op_ty,
            });
        }

        CHILD_NODE_INDICES.set(Some(child_node_indices));
        NODE_INDICES.set(Some(node_indices));
    }

    pub fn propagate_tuple_from_u32_op(node_idx: usize, snarl: &mut Snarl<Self>) {
        thread_local! {
            static CHILD_NODE_INDICES: RefCell<Option<HashSet<usize>>> = RefCell::new(Some(Default::default()));
            static NODE_INDICES: RefCell<Option<Vec<usize>>> = RefCell::new(Some(Default::default()));
        }

        let mut child_node_indices = CHILD_NODE_INDICES.take().unwrap();
        let mut node_indices = NODE_INDICES.take().unwrap();
        node_indices.push(node_idx);

        while let Some(node_idx) = node_indices.pop() {
            if child_node_indices.insert(node_idx) {
                if let node @ Self::U32Operation(_) = snarl.get_node(node_idx) {
                    let op = node.as_const_op_u32().unwrap();
                    node_indices.extend(op.inputs.iter().filter_map(|input| input.as_node_index()));
                    node_indices.extend(
                        snarl
                            .out_pin(OutPinId {
                                node: node_idx,
                                output: 0,
                            })
                            .remotes
                            .iter()
                            .map(|remote| remote.node),
                    );
                } else {
                    child_node_indices.clear();
                    CHILD_NODE_INDICES.set(Some(child_node_indices));

                    node_indices.clear();
                    NODE_INDICES.set(Some(node_indices));

                    return;
                }
            }
        }

        for node_idx in child_node_indices.drain() {
            let node = snarl.get_node_mut(node_idx);
            let op = node.as_const_op_u32().unwrap().clone();

            *node = NoiseNode::Operation(ConstantOpNode {
                inputs: op
                    .inputs
                    .iter()
                    .copied()
                    .map(|input| {
                        input
                            .as_node_index()
                            .map(NodeValue::Node)
                            .unwrap_or_default()
                    })
                    .collect::<Vec<_>>()
                    .try_into()
                    .unwrap(),
                op_ty: op.op_ty,
            });
        }

        CHILD_NODE_INDICES.set(Some(child_node_indices));
        NODE_INDICES.set(Some(node_indices));
    }

    pub fn propagate_u32_from_tuple_op(node_idx: usize, snarl: &mut Snarl<Self>) {
        thread_local! {
            static CHILD_NODE_INDICES: RefCell<Option<HashSet<usize>>> = RefCell::new(Some(Default::default()));
            static NODE_INDICES: RefCell<Option<Vec<usize>>> = RefCell::new(Some(Default::default()));
        }

        let mut child_node_indices = CHILD_NODE_INDICES.take().unwrap();
        let mut node_indices = NODE_INDICES.take().unwrap();
        node_indices.push(node_idx);

        while let Some(node_idx) = node_indices.pop() {
            if child_node_indices.insert(node_idx) {
                node_indices.extend(
                    snarl
                        .out_pin(OutPinId {
                            node: node_idx,
                            output: 0,
                        })
                        .remotes
                        .iter()
                        .map(|remote| remote.node),
                );

                if let node @ Self::Operation(_) = snarl.get_node_mut(node_idx) {
                    let op = node.as_const_op_tuple().unwrap().clone();
                    node_indices.extend(op.inputs.iter().filter_map(|input| input.as_node_index()));

                    *node = NoiseNode::U32Operation(ConstantOpNode {
                        inputs: op
                            .inputs
                            .iter()
                            .copied()
                            .map(|input| {
                                input
                                    .as_node_index()
                                    .map(NodeValue::Node)
                                    .unwrap_or_default()
                            })
                            .collect::<Vec<_>>()
                            .try_into()
                            .unwrap(),
                        op_ty: op.op_ty,
                    });
                } else {
                    unreachable!();
                }
            }
        }

        child_node_indices.clear();
        CHILD_NODE_INDICES.set(Some(child_node_indices));
        NODE_INDICES.set(Some(node_indices));
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct RigidFractalNode {
    pub image: Image,

    pub source_ty: SourceType,
    pub seed: NodeValue<u32>,
    pub octaves: NodeValue<u32>,
    pub frequency: NodeValue<f64>,
    pub lacunarity: NodeValue<f64>,
    pub persistence: NodeValue<f64>,
    pub attenuation: NodeValue<f64>,
}

impl RigidFractalNode {
    fn expr(&self, snarl: &Snarl<NoiseNode>) -> RigidFractalExpr {
        RigidFractalExpr {
            source_ty: self.source_ty,
            seed: self.seed.var(snarl),
            octaves: self.octaves.var(snarl),
            frequency: self.frequency.var(snarl),
            lacunarity: self.lacunarity.var(snarl),
            persistence: self.persistence.var(snarl),
            attenuation: self.attenuation.var(snarl),
        }
    }
}

impl Default for RigidFractalNode {
    fn default() -> Self {
        Self {
            image: Default::default(),
            source_ty: Default::default(),
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

    pub scale: NodeValue<f64>,
    pub bias: NodeValue<f64>,
}

impl ScaleBiasNode {
    fn expr(&self, snarl: &Snarl<NoiseNode>) -> ScaleBiasExpr {
        ScaleBiasExpr {
            source: self
                .input_node_idx
                .map(|node_idx| Box::new(snarl.get_node(node_idx).expr(snarl)))
                .unwrap_or_else(|| constant(0.0)),
            scale: self.scale.var(snarl),
            bias: self.bias.var(snarl),
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct SelectNode {
    pub image: Image,

    pub input_node_indices: [Option<usize>; 2],
    pub control_node_idx: Option<usize>,

    pub lower_bound: NodeValue<f64>,
    pub upper_bound: NodeValue<f64>,
    pub falloff: NodeValue<f64>,
}

impl SelectNode {
    fn expr(&self, snarl: &Snarl<NoiseNode>) -> SelectExpr {
        SelectExpr {
            sources: self
                .input_node_indices
                .iter()
                .map(|node_idx| {
                    node_idx
                        .map(|node_idx| Box::new(snarl.get_node(node_idx).expr(snarl)))
                        .unwrap_or_else(|| constant(0.0))
                })
                .collect::<Vec<_>>()
                .try_into()
                .unwrap(),
            control: self
                .control_node_idx
                .map(|node_idx| Box::new(snarl.get_node(node_idx).expr(snarl)))
                .unwrap_or_else(|| constant(0.0)),
            lower_bound: self.lower_bound.var(snarl),
            upper_bound: self.upper_bound.var(snarl),
            falloff: self.falloff.var(snarl),
        }
    }
}

impl Default for SelectNode {
    fn default() -> Self {
        Self {
            image: Default::default(),
            input_node_indices: Default::default(),
            control_node_idx: Default::default(),
            lower_bound: NodeValue::Value(0.0),
            upper_bound: NodeValue::Value(1.0),
            falloff: NodeValue::Value(0.0),
        }
    }
}

impl Default for SourceType {
    fn default() -> Self {
        Self::Perlin
    }
}

#[derive(Clone, Default, Serialize, Deserialize)]
pub struct TerraceNode {
    pub image: Image,

    pub input_node_idx: Option<usize>,

    pub inverted: bool,
    pub control_point_node_indices: Vec<Option<usize>>,
}

impl TerraceNode {
    fn expr(&self, snarl: &Snarl<NoiseNode>) -> TerraceExpr {
        TerraceExpr {
            source: self
                .input_node_idx
                .map(|node_idx| Box::new(snarl.get_node(node_idx).expr(snarl)))
                .unwrap_or_else(|| constant(0.0)),
            inverted: self.inverted,
            control_points: self
                .control_point_node_indices
                .iter()
                .copied()
                .filter_map(|node_idx| {
                    node_idx.map(|node_idx| match snarl.get_node(node_idx) {
                        NoiseNode::F64(node) => Variable::Named(node.name.clone(), node.value),
                        NoiseNode::F64Operation(node) => node.var(snarl),
                        _ => unreachable!(),
                    })
                })
                .collect(),
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct TransformNode {
    pub image: Image,

    pub input_node_idx: Option<usize>,

    pub axes: [NodeValue<f64>; 4],
}

impl TransformNode {
    fn new(value: f64) -> Self {
        Self {
            image: Default::default(),
            input_node_idx: Default::default(),
            axes: [NodeValue::Value(value); 4],
        }
    }

    fn expr(&self, snarl: &Snarl<NoiseNode>) -> TransformExpr {
        TransformExpr {
            source: self
                .input_node_idx
                .map(|node_idx| Box::new(snarl.get_node(node_idx).expr(snarl)))
                .unwrap_or_else(|| constant(0.0)),
            axes: self
                .axes
                .iter()
                .map(|axis| axis.var(snarl))
                .collect::<Vec<_>>()
                .try_into()
                .unwrap(),
        }
    }

    pub fn one() -> Self {
        Self::new(1.0)
    }

    pub fn zero() -> Self {
        Self::new(0.0)
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct TurbulenceNode {
    pub image: Image,

    pub input_node_idx: Option<usize>,

    pub source_ty: SourceType,
    pub seed: NodeValue<u32>,
    pub frequency: NodeValue<f64>,
    pub power: NodeValue<f64>,
    pub roughness: NodeValue<u32>,
}

impl TurbulenceNode {
    fn expr(&self, snarl: &Snarl<NoiseNode>) -> TurbulenceExpr {
        TurbulenceExpr {
            source: self
                .input_node_idx
                .map(|node_idx| Box::new(snarl.get_node(node_idx).expr(snarl)))
                .unwrap_or_else(|| constant(0.0)),
            source_ty: self.source_ty,
            seed: self.seed.var(snarl),
            frequency: self.frequency.var(snarl),
            power: self.power.var(snarl),
            roughness: self.roughness.var(snarl),
        }
    }
}

impl Default for TurbulenceNode {
    fn default() -> Self {
        Self {
            image: Default::default(),
            input_node_idx: Default::default(),
            source_ty: Default::default(),
            seed: NodeValue::Value(Turbulence::<AnySeedable, AnySeedable>::DEFAULT_SEED),
            frequency: NodeValue::Value(Turbulence::<AnySeedable, AnySeedable>::DEFAULT_FREQUENCY),
            power: NodeValue::Value(Turbulence::<AnySeedable, AnySeedable>::DEFAULT_POWER),
            roughness: NodeValue::Value(
                Turbulence::<AnySeedable, AnySeedable>::DEFAULT_ROUGHNESS as _,
            ),
        }
    }
}

#[derive(Clone, Default, Serialize, Deserialize)]
pub struct UnaryNode {
    pub image: Image,

    pub input_node_idx: Option<usize>,
}

impl UnaryNode {
    fn expr(&self, snarl: &Snarl<NoiseNode>) -> Box<Expr> {
        self.input_node_idx
            .map(|node_idx| Box::new(snarl.get_node(node_idx).expr(snarl)))
            .unwrap_or_else(|| constant(0.0))
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct WorleyNode {
    pub image: Image,

    pub seed: NodeValue<u32>,
    pub frequency: NodeValue<f64>,
    pub distance_fn: DistanceFunction,
    pub return_ty: ReturnType,
}

impl WorleyNode {
    fn expr(&self, snarl: &Snarl<NoiseNode>) -> WorleyExpr {
        WorleyExpr {
            seed: self.seed.var(snarl),
            frequency: self.frequency.var(snarl),
            distance_fn: self.distance_fn,
            return_ty: self.return_ty,
        }
    }
}

impl Default for WorleyNode {
    fn default() -> Self {
        Self {
            image: Default::default(),
            seed: NodeValue::Value(Worley::DEFAULT_SEED),
            frequency: NodeValue::Value(Worley::DEFAULT_FREQUENCY),
            distance_fn: DistanceFunction::Euclidean,
            return_ty: ReturnType::Value,
        }
    }
}
