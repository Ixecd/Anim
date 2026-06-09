fix: 豆包review——Pass 6 三漏洞修复 + FORGET 新增4项

- recover.rs damped_main 双向查询：Emotional→Visceral || Visceral→Emotional
- recover.rs damping_active 去掉strategy条件：异常Session也应用阻尼
- registry.rs max_ratio [0.0,1.0] + NaN/Infinity 校验（外部JSON）
- main.rs 未知参数静默吞噬→warn
- FORGET P1 #22 Registry哈希依赖原子顺序、P1 #23 基线偏移硬编码情绪维度、
  P2 #24 log.rs Relaxed→Release/Acquire
- 96 tests pass, 0 warnings