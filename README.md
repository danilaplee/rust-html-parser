# TGNEWS

## The binary is already in the folder - ./tgnews

### A telegram competition nlu news parser written in rust


this script accepts a folder of html files and finds languages, news, categories, threads.

```
cargo build --bin tgnews --release && mv target/release/tgnews ./

./tgnews debug ./DataClusteringSample0817

./tgnews languages ./DataClusteringSample0817

./tgnews news ./DataClusteringSample0817

./tgnews categories ./DataClusteringSample0817

./tgnews threads ./DataClusteringSample0817

./tgnews top ./DataClusteringSample0817

```


to run the python bert environment, install the following:

```
	apt-get install redis
	apt-get install python3
	apt-get install pip3
	pip3 install deeppavlov

```