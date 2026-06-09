fix: 解除DampingMatrix死锁+四维系数全通+点缀比例帽从Registry驱动

- permanent.rs: _damping → damping_gradients: Option
  v0.3 无传感器数据 → None → 阻尼关闭。DampingMatrix不再架空死锁。
- permanent.rs: 四维系数打通——AtomEntry新增dimension字段。
  从Registry查询每个原子的主维度，对号选用四维系数。
  8个内建原子: calm_meditative=Emotional, deep_rest=Visceral, warmth=Tactile等。
- permanent.rs: 点缀比例帽从default_accent_cap()改为registry.max_ratio()驱动。
- registry.rs: AtomEntry新增dimension: PbmDimension, AtomDef支持serde(default=Emotional)。
- FORGET P1#23/#25/#26全闭合。
- 99 tests pass, 0 warnings.