from deeppavlov import build_model, configs
import redis
import time
import traceback
import numpy as np

r = redis.Redis(host='localhost', port=6379, db=0, decode_responses=True)

PIPELINE_CONFIG_PATH = configs.ner.ner_ontonotes_bert_mult

model = build_model(PIPELINE_CONFIG_PATH, download=False)

def RedisListener():
    try:

        p = r.pubsub()                                                             
        p.subscribe('tgnews_ner')
        r.publish("tgnews_ner", "listener_started")                                                 
        PAUSE = True

        while PAUSE:                                    
            message = p.get_message()                                               
            if message:
                command = message['data'] 
                print("NEW MESSAGE FOR REDIS LISTENER", command, model([str(command)]))                                             

    except Exception as e:
        print("!!!!!!!!!! EXCEPTION !!!!!!!!!")
        print(str(e))
        print(traceback.format_exc())

RedisListener()
print('======= AgNerService started =======')