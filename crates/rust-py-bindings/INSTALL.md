# Polars Greeks 安装指南

高性能期权Greeks计算插件，基于Rust实现的Polars扩展。

## 快速安装

### 方式1：GitLab Package Registry (推荐) ⚡

```bash
# 配置pip源 (一次性配置)
pip config set global.extra-index-url https://git.signalpluslab.com/api/v4/projects/59/packages/pypi/simple
pip config set global.trusted-host git.signalpluslab.com

# 安装预编译包 (无需Rust工具链)
pip install polars-greeks
```

或者在requirements.txt中：
```txt
--extra-index-url https://git.signalpluslab.com/api/v4/projects/59/packages/pypi/simple
--trusted-host git.signalpluslab.com
polars>=0.20.0
polars-greeks>=0.1.0
```

### 方式2：Git直接安装 (开发用)

```bash
pip install git+ssh://git@git.signalpluslab.com/derek.jiao/rust-py.git#subdirectory=crates/rust-py-bindings
```

### 方式3：本地开发安装

```bash
# 克隆仓库
git clone git@git.signalpluslab.com:derek.jiao/rust-py.git
cd rust-py/crates/rust-py-bindings

# 开发模式安装
pip install -e .
```

## 系统要求

### GitLab Package Registry方式 (推荐)
- **Python**: >=3.8
- **依赖**: polars>=0.20.0
- **网络**: 访问内网GitLab
- **无需Rust工具链** ✅

### Git/本地安装方式
- **Python**: >=3.8  
- **Rust工具链**: 必须安装Rust (用于源码编译)
- **依赖**: polars>=0.20.0

#### 安装Rust工具链

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env
```

## 使用示例

```python
import polars as pl
import polars_greeks as greeks

# 创建期权数据
df = pl.DataFrame({
    "spot": [100.0, 105.0, 95.0],
    "strike": [100.0, 100.0, 100.0], 
    "time_to_expiry": [0.25, 0.25, 0.25],
    "volatility": [0.2, 0.2, 0.2],
    "is_call": [True, False, True]
})

# 计算Greeks
result = df.with_columns([
    greeks.df(
        pl.col("spot"),
        strike=pl.col("strike"),
        time_to_expiry=pl.col("time_to_expiry"),
        volatility=pl.col("volatility"),
        r=pl.lit(0.05),      # 无风险利率 - 注意使用pl.lit()
        q=pl.lit(0.0),       # 股息率 - 注意使用pl.lit()
        is_call=pl.col("is_call"),
        greeks=["vega", "delta", "gamma", "theta"]
    ).alias("all_greeks")
]).with_columns([
    pl.col("all_greeks").struct.field("vega").alias("vega"),
    pl.col("all_greeks").struct.field("delta").alias("delta"),
    pl.col("all_greeks").struct.field("gamma").alias("gamma"),
    pl.col("all_greeks").struct.field("theta").alias("theta"),
])

print(result)
```

## 重要说明

⚠️ **常量参数必须使用 `pl.lit()`包装**
- ✅ 正确：`r=pl.lit(0.05), q=pl.lit(0.0)`
- ❌ 错误：`r=0.05, q=0.0`

这是Polars插件系统的限制，所有参数都必须是表达式类型。

## 支持的Greeks

- **delta**: 价格敏感性
- **gamma**: delta的敏感性  
- **theta**: 时间衰减
- **vega**: 波动率敏感性
- **rho**: 利率敏感性
- **vanna**: delta对波动率的敏感性
- **volga**: vega对波动率的敏感性
- **charm**: delta对时间的敏感性
- **speed**: gamma对价格的敏感性
- **zomma**: gamma对波动率的敏感性

## 性能特性

- 🚀 **零拷贝数据处理**: 直接在Polars Series上操作
- ⚡ **批量计算优化**: 单次遍历计算多个Greeks
- 🎯 **选择性计算**: 只计算请求的Greeks，避免不必要计算
- 💾 **内存高效**: 预分配向量，避免重复分配

## 故障排除

### 编译错误
- 确保安装了Rust工具链
- 检查Polars版本兼容性

### 运行时错误  
- 确保常量使用`pl.lit()`包装
- 检查数据类型匹配

## 发布新版本 (维护者使用)

GitLab CI会自动构建和发布：

```bash
# 更新版本号
vim crates/rust-py-bindings/pyproject.toml  # 修改version = "x.y.z"
vim crates/rust-py-bindings/python/polars_greeks/__init__.py  # 修改__version__

# 提交并打tag
git add -A
git commit -m "Release v0.1.1"  
git tag v0.1.1
git push origin main --tags

# GitLab CI会自动构建wheel并上传到Package Registry
```

查看发布状态：
- **CI/CD页面**: https://git.signalpluslab.com/derek.jiao/rust-py/-/pipelines
- **包列表页面**: https://git.signalpluslab.com/derek.jiao/rust-py/-/packages

## 项目信息

- **仓库**: https://git.signalpluslab.com/derek.jiao/rust-py
- **Package Registry**: https://git.signalpluslab.com/derek.jiao/rust-py/-/packages
- **作者**: derek.jiao@signalplus.com  
- **许可**: MIT