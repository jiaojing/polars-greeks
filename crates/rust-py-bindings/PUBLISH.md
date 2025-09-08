# PyPI 发布指南

## 发布前准备

### 1. 更新配置
编辑 `pyproject.toml`，替换占位符：
```toml
[project.urls]
Homepage = "https://github.com/YOUR_GITHUB_USERNAME/polars-greeks"
Repository = "https://github.com/YOUR_GITHUB_USERNAME/polars-greeks"
Issues = "https://github.com/YOUR_GITHUB_USERNAME/polars-greeks/issues"
```

### 2. 注册PyPI账号
- 访问 https://pypi.org 注册账号
- 访问 https://test.pypi.org 注册测试账号（建议先在测试环境验证）

### 3. 配置API Token
```bash
# 在PyPI账号设置中创建API token，然后配置
pip install twine
```

## 发布流程

### 步骤1：构建wheel包
```bash
# 激活虚拟环境
source .venv/bin/activate

# 构建release版本wheel包
maturin build --release

# 检查生成的wheel文件
ls target/wheels/
```

### 步骤2：测试发布到TestPyPI
```bash
# 上传到测试PyPI
twine upload --repository-url https://test.pypi.org/legacy/ target/wheels/*

# 测试安装
pip install --index-url https://test.pypi.org/simple/ --extra-index-url https://pypi.org/simple/ polars-greeks

# 验证功能
python -c "import polars_greeks as greeks; print(greeks.scalar(100, 100, 0.25, 0.2))"
```

### 步骤3：发布到正式PyPI
```bash
# 确认测试无误后，发布到正式PyPI
twine upload target/wheels/*

# 验证正式安装
pip install polars-greeks
```

## 自动化发布（可选）

### GitHub Actions配置
创建 `.github/workflows/publish.yml`：

```yaml
name: Publish to PyPI

on:
  release:
    types: [published]

jobs:
  build-and-publish:
    runs-on: ubuntu-latest
    
    steps:
    - uses: actions/checkout@v3
    
    - name: Set up Python
      uses: actions/setup-python@v4
      with:
        python-version: '3.9'
    
    - name: Install Rust
      uses: actions-rs/toolchain@v1
      with:
        toolchain: stable
        
    - name: Install maturin
      run: pip install maturin[patchelf]
    
    - name: Build wheels
      run: maturin build --release --out dist/
      working-directory: crates/rust-py-bindings
    
    - name: Publish to PyPI
      env:
        TWINE_USERNAME: __token__
        TWINE_PASSWORD: ${{ secrets.PYPI_API_TOKEN }}
      run: |
        pip install twine
        twine upload dist/*
```

## 版本管理

### 更新版本号
编辑 `pyproject.toml` 和 `__init__.py` 中的版本号：
```toml
version = "0.1.1"  # 遵循语义化版本控制
```

### 创建Release
```bash
git tag v0.1.0
git push origin v0.1.0

# 在GitHub上创建Release
```

## 验证发布

### 检查包信息
```bash
# 查看包信息
pip show polars-greeks

# 检查PyPI页面
# https://pypi.org/project/polars-greeks/
```

### 功能测试
```python
import polars as pl
import polars_greeks as greeks

# DataFrame测试
df = pl.DataFrame({
    "spot": [100.0], 
    "strike": [100.0],
    "time_to_expiry": [0.25],
    "volatility": [0.2],
    "is_call": [True]
})

result = df.select(
    greeks.df(
        pl.col("spot"),
        strike=pl.col("strike"),
        time_to_expiry=pl.col("time_to_expiry"),
        volatility=pl.col("volatility"),
        r=pl.lit(0.05),
        q=pl.lit(0.0), 
        is_call=pl.col("is_call"),
        greeks=["delta", "vega"]
    ).alias("greeks")
)

print("DataFrame API:", result)

# 标量测试
scalar_result = greeks.scalar(
    spot=100.0,
    strike=100.0,
    time_to_expiry=0.25,
    volatility=0.2,
    greeks=["delta", "vega"]
)

print("Scalar API:", scalar_result)
```

## 故障排除

### 常见问题
1. **构建失败** - 确保Rust工具链正确安装
2. **上传失败** - 检查API token配置
3. **包名冲突** - 确认包名唯一性
4. **版本冲突** - 每次发布必须使用新版本号

### 依赖问题
- 确保 `polars>=0.20.0` 兼容性
- 测试不同Python版本（3.8-3.12）

## 推广

### 文档更新
- 更新 README.md
- 添加使用示例
- 编写API文档

### 社区推广
- 在相关论坛/社区分享
- 撰写技术博客
- 参与开源社区讨论