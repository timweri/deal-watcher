FROM python:3

RUN apt-get update    
RUN apt-get install -y cron

WORKDIR /usr/src/app

COPY requirements.txt ./

RUN pip install --no-cache-dir -r requirements.txt

COPY *.py .

CMD [ "python", "cron.py"]
