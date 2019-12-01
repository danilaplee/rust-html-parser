from deeppavlov import build_model, configs
import redis
import time
import traceback
import json
import numpy as np
import pprint

r = redis.Redis(host='localhost', port=6379, db=0, decode_responses=True)

# PIPELINE_CONFIG_PATH = configs.ner.ner_ontonotes_bert

# model = build_model(PIPELINE_CONFIG_PATH, download=False)
# print("======== model loaded, listing vocabulary: ===========")
# print(list(model["tag_vocab"].items()))

english_documents = r.smembers("news")
pp = pprint.PrettyPrinter(depth=6)
data = {}
data_processed = {}

for document in english_documents:
    j = json.loads(document)
    for wclass in j["words"].keys():
        
        if wclass == "":
            continue;

        if wclass not in data:
            data[wclass] = j["words"][wclass];
        else: 
            data[wclass] += j["words"][wclass];


for key in data.keys():
    
    d = list()
    for w in data[key]:
        if ((len(w) >= 3) and type(w) is str):
            d.append(w);

    if key not in data_processed:
        data_processed[key] = d;


pp.pprint(data_processed);


filehandle = open('dict4.json', 'w')
filehandle.write(json.dumps(data_processed))
filehandle.close()