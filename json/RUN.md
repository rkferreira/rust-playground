# RUN test

```
for i in tests/step?/* ; do echo $i ; ./target/release/json --name $i ; echo -e "\n" ;done
```
