from setuptools import setup, Extension
from setuptools.command.build_ext import build_ext
import sys
import os
import pybind11

# 获取pybind11的包含目录
pybind11_include_dir = pybind11.get_include()

# 添加remdb核心库的包含目录和库文件路径
remdb_include_dir = os.path.abspath('../remdb/include')
remdb_lib_dir = os.path.abspath('../remdb/target/release')

# 定义C扩展模块
ext_modules = [
    Extension(
        '_remdb',
        sources=['src/remdb_extension.cpp'],
        include_dirs=['src', pybind11_include_dir, remdb_include_dir],
        library_dirs=[remdb_lib_dir],
        libraries=['remdb', 'ws2_32', 'advapi32', 'kernel32', 'ntdll', 'bcrypt', 'userenv'],
        language='c++'
    )
]

# 自定义构建扩展类
class BuildExt(build_ext):
    def build_extensions(self):
        # 确保使用C++11或更高版本
        for ext in self.extensions:
            if ext.language == 'c++':
                # 为不同编译器设置合适的C++标准选项
                if sys.platform == 'win32':
                    # MSVC使用不同的语法
                    ext.extra_compile_args = ['/std:c++11']
                else:
                    # GCC/Clang
                    ext.extra_compile_args = ['-std=c++11']
        super().build_extensions()

# 读取README.md作为长描述
try:
    with open('README.md', 'r', encoding='utf-8') as f:
        long_description = f.read()
except FileNotFoundError:
    long_description = ''

setup(
    name='remdb-python',
    version='0.1.0',
    description='Python bindings for RemDB embedded database',
    long_description=long_description,
    long_description_content_type='text/markdown',
    author='RemDB Team',
    author_email='remdb@example.com',
    url='https://github.com/remdb/remdb-python',
    packages=['remdb', 'remdb.extras'],
    package_dir={'remdb': 'remdb'},
    ext_modules=ext_modules,
    cmdclass={'build_ext': BuildExt},
    install_requires=[
        'pybind11>=2.6.0'
    ],
    extras_require={
        'numpy': ['numpy>=1.18.0'],
        'pandas': ['pandas>=1.0.0'],
        'async': ['asyncio>=3.4.3']
    },
    python_requires='>=3.8',
    classifiers=[
        'Development Status :: 3 - Alpha',
        'Intended Audience :: Developers',
        'License :: OSI Approved :: MIT License',
        'Programming Language :: Python :: 3',
        'Programming Language :: Python :: 3.8',
        'Programming Language :: Python :: 3.9',
        'Programming Language :: Python :: 3.10',
        'Programming Language :: Python :: 3.11',
        'Programming Language :: Python :: 3.12',
        'Topic :: Database',
        'Topic :: Database :: Database Engines/Servers',
        'Topic :: Software Development :: Libraries :: Python Modules'
    ],
    license='MIT',
    keywords='remdb embedded database python bindings',
    project_urls={
        'Documentation': 'https://remdb.readthedocs.io/',
        'Source': 'https://github.com/remdb/remdb-python',
        'Tracker': 'https://github.com/remdb/remdb-python/issues'
    }
)
