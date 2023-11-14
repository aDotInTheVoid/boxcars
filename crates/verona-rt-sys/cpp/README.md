```
cmake -Bbuild -GNinja -DCMAKE_BUILD_TYPE=Debug
cmake -Bbuild-release -GNinja -DCMAKE_BUILD_TYPE=Release
ninja -C ./build-release/ libboxcar_bindings.a 
```