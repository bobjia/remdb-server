@echo off

rem 检查是否存在带依赖的JAR文件
if not exist "target\remdb-jdbc-driver-0.2.0-jar-with-dependencies.jar" (
    echo 正在构建带所有依赖的JAR文件...
    mvn package -DskipTests
)

echo 正在运行RemDb向量功能测试...
java -cp target\remdb-jdbc-driver-0.2.0-jar-with-dependencies.jar cn.totaltrust.remdb.RemDbVectorExample

pause