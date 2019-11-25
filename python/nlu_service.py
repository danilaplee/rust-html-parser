from deeppavlov import build_model, configs
import redis
import time
import traceback
import json
import numpy as np

r = redis.Redis(host='localhost', port=6379, db=0, decode_responses=True)

PIPELINE_CONFIG_PATH = configs.ner.ner_ontonotes_bert

model = build_model(PIPELINE_CONFIG_PATH, download=False)
tgnews_nlu_reply = "tgnews_nlu_reply_list"
tgnews_nlu_start = "tgnews_nlu_start"
tgnews_nlu_end = "tgnews_nlu_end"
tgnews_nlu = "tgnews_nlu"
def RedisListener():
    try:

        p = r.pubsub()                                                             
        p.subscribe(tgnews_nlu)
        r.publish(tgnews_nlu_start, "listener_started")                                                 
        PAUSE = True

        while PAUSE:                                    
            message = p.get_message()                                               
            if message:
                # print("new message {}", message["data"])
                try:
                    data = json.loads(str(message['data']))
                    # print('==== json data =====', data)

                    data['response'] = model([str(data['h1'])])
                    response = json.dumps(data)
                    # print("sending reply {}", response)
                    r.lpush(tgnews_nlu_reply, response);
                except Exception:
                    print(traceback.print_exc())
                    continue;                                       

    except Exception as e:
        print("!!!!!!!!!! NLU EXCEPTION !!!!!!!!!")
        print(str(e))
        print(traceback.format_exc())

RedisListener()

print('======= NLU SERVICE STARTED =======')