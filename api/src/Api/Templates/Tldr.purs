module Api.Templates.Classification where

import Prelude

import Data.String (trim)

classificationTemplate :: String -> String
classificationTemplate snippet = """
Q: 
```
from flask import Flask
app = Flask(__name__)
@app.route("/")
def hello():
    return "Hello, World!"
```
A: http
###
Q:
```
>>> import simplejson as json
>>> json.dumps(['foo', {'bar': ('baz', None, 1.0, 2)}])
'["foo", {"bar": ["baz", null, 1.0, 2]}]'
```
A: json
###
Q: 
```
from tensorflow.keras.models import Sequential

model = Sequential()
from tensorflow.keras.layers import Dense

model.add(Dense(units=64, activation='relu'))
model.add(Dense(units=10, activation='softmax'))
```
A: machine-learning
###
Q: """ <> trim snippet <> "\nA: "
