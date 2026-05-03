首先是读文档，了解任务是什么如何判题

ch2没有TODO，然后就是构建docker 运行起来看一下

```bash

make build-docker 
make  docker
# entry container
cd os && make run 
```

## 然后看README.md 发现评测逻辑，录取一个ci的测评库，进行构建评测
主要是比较os的输出有没有出现特定字符串和以及是否出现了不该出现的字符串


ch2的话，大概就是把4个程序给补上，在补上链接脚本，然后编译链接一下就好了，这个在rcore的那本书里面有，
大概看一下就可以知道该干嘛。

因为在wsl使用的是普通用户,所有要使用sudo -E bash