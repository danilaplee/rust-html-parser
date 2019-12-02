# TGNEWS

## The binary for debian is already in the folder - ./tgnews

### A telegram competition nlu news parser written in rust


this script accepts a folder of html files and finds languages, news, categories, threads and top.

```

./tgnews debug ./DataClusteringSample0817

./tgnews languages ./DataClusteringSample0817

./tgnews news ./DataClusteringSample0817

./tgnews categories ./DataClusteringSample0817

./tgnews threads ./DataClusteringSample0817

./tgnews top ./DataClusteringSample0817


#script will show 0 results if directory path is invalid
```


to run the optional python bert environment, do the following:

```
	apt-get install redis
	apt-get install python3
	apt-get install pip3
	pip3 install deeppavlov
    ./tgnews news ./DataClusteringSample0817 --python
```


to build this project with cargo, first rustup and select nightly:
```
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh 
```

then just build with cargo:
```
cargo build --bin tgnews --release && mv target/release/tgnews ./
```
