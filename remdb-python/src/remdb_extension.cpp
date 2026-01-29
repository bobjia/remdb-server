#include <pybind11/pybind11.h>

namespace py = pybind11;

// 绑定到Python
PYBIND11_MODULE(_remdb, m) {
    m.doc() = "RemDB Python bindings";
    
    // 绑定简单的常量
    m.attr("__version__") = "0.1.0";
    m.attr("connected") = false;
    
    // 绑定简单的函数
    m.def("test", []() {
        return "RemDB Python bindings initialized successfully!";
    });
}
