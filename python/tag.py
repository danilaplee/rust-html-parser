from deeppavlov import build_model

model = build_model("topic_ag_news", download=True)

# get predictions for 'input_text1', 'input_text2'
predictions = model([
	'Britain Prince Andrew has no recollection of meeting sex accuser Report',
	'Maher panics says Nikki Haley has gone full on Team Deplorable This is so scary to me',
	'Government must act after Glasgow hospital water infection scandal',
	'More excited to represent Wales than Real Madrid says Gareth Bale',
	'Joker about to cross $1 billion global box office milestone',
	'New Pixel reacts to gestures, but charge still short',
	'LETTER On vets, vettes and other vets',
	'Apple TV See Is a Great Show that Transcends Its Captivating Premise',
	'Google is scaling back its weekly all hands meetings after leaks Sundar Pichai tells staff',
	'Czechs use anniversary of Velvet Revolution to pressure PM',
	'Everything You Need To Watch On Netflix Tonight',
	'Learn how to make your own video games with this online course',
	'Hunters head out for start of deer firearms season in northeast Missouri',
	'The HUAWEI FreeBuds 3 with Intelligent Noise Cancellation',
	'Some of the protesters targetted cars and shop windows during the clashes',
	'Thomas leads Appalachian St. to 56-27 win over Georgia St.'
])
print("predictions: ", predictions)