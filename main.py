from tensorflow.keras.models import load_model
from tensorflow.keras.preprocessing.text import Tokenizer
from tensorflow.keras.preprocessing.sequence import pad_sequences
from sklearn.preprocessing import LabelEncoder
import re
import nltk
from nltk.corpus import stopwords
import argparse
from nltk.stem import SnowballStemmer, WordNetLemmatizer
# nltk.download("stopwords")
stop_words = set(stopwords.words("english"))

# parser = argparse.ArgumentParser(description="A sample script demonstrating argparse.")
# parser.add_argument("--name", type=str, help="Your name")
# parser.add_argument("--age", type=int, default=30, help="Your age")


import numpy as np
import pandas as pd
import sys

# print(sys.argv)


def lemmatization(text):
    lemmatizer= WordNetLemmatizer()

    text = text.split()

    text=[lemmatizer.lemmatize(y) for y in text]
    
    return " " .join(text)

def remove_stop_words(text):

    Text=[i for i in str(text).split() if i not in stop_words]
    return " ".join(Text)

def Removing_numbers(text):
    text=''.join([i for i in text if not i.isdigit()])
    return text

def lower_case(text):
    
    text = text.split()

    text=[y.lower() for y in text]
    
    return " " .join(text)

def Removing_punctuations(text):
    ## Remove punctuations
    text = re.sub('[%s]' % re.escape("""!"#$%&'()*+,،-./:;<=>؟?@[\]^_`{|}~"""), ' ', text)
    text = text.replace('؛',"", )
    
    ## remove extra whitespace
    text = re.sub('\s+', ' ', text)
    text =  " ".join(text.split())
    return text.strip()

def Removing_urls(text):
    url_pattern = re.compile(r'https?://\S+|www\.\S+')
    return url_pattern.sub(r'', text)

def remove_small_sentences(df):
    for i in range(len(df)):
        if len(df.text.iloc[i].split()) < 3:
            df.text.iloc[i] = np.nan
            
def normalize_text(df):
    df.Text=df.Text.apply(lambda text : lower_case(text))
    df.Text=df.Text.apply(lambda text : remove_stop_words(text))
    df.Text=df.Text.apply(lambda text : Removing_numbers(text))
    df.Text=df.Text.apply(lambda text : Removing_punctuations(text))
    df.Text=df.Text.apply(lambda text : Removing_urls(text))
    df.Text=df.Text.apply(lambda text : lemmatization(text))
    return df

def normalized_sentence(sentence):
    sentence= lower_case(sentence)
    sentence= remove_stop_words(sentence)
    sentence= Removing_numbers(sentence)
    sentence= Removing_punctuations(sentence)
    sentence= Removing_urls(sentence)
    sentence= lemmatization(sentence)
    return sentence

le = LabelEncoder()


# Read datasets
data = pd.read_csv('data2.csv', names=['Text', 'Emotion'], skiprows=1)
data_val = pd.read_csv('data_val.csv', names=['Text', 'Emotion'],skiprows=1)
data_test = pd.read_csv('data_test.csv', names=['Text', 'Emotion'],skiprows=1)

#Splitting the text from the labels
X_train = data['Text']
y_train = data['Emotion']

X_test = data_test['Text']
y_test = data_test['Emotion']

# Encode labels
le = LabelEncoder()
y_train = le.fit_transform(y_train)
y_test = le.transform(y_test)

model = load_model('Emotion Recognition From English text.h5')
# print("Model loaded successfully!")

tokenizer = Tokenizer(oov_token="UNK")
tokenizer.fit_on_texts(pd.concat([X_train, X_test], axis=0))

# print("Tokenizer ready")

sentence= sys.argv[1]
# sentence = sentence.replace("\n", "")
# print(sentence)
sentence = normalized_sentence(sentence)
sentence = tokenizer.texts_to_sequences([sentence])
sentence = pad_sequences(sentence, maxlen=229, truncating='pre')
result = le.inverse_transform(np.argmax(model.predict(sentence), axis=-1))[0]
proba =  np.max(model.predict(sentence))
# print(f"{result} : {proba}\n\n")
print(f"emotion_detected:{result}");


# args = parser.parse_args()

# print(f"Sentence: {sentence}")
# print(f"Name: {args.name}")
# print(f"Age: {args.age}")