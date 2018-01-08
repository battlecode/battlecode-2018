FROM anapsix/alpine-java

RUN apk add --update --no-cache \
    python-dev \
    py-pip \
    build-base \
    python3-dev && \
    python3 -m ensurepip && \
    rm -r /usr/lib/python*/ensurepip && \
    pip3 install --upgrade pip setuptools && \
    rm -r /root/.cache
