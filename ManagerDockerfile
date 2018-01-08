FROM docker:18.01.0-ce-rc1-dind

# Install GCC
RUN apk add --no-cache gcc musl-dev

# Install Python3
RUN apk add --update --no-cache \
    build-base \
    python3-dev && \
    python3 -m ensurepip && \
    rm -r /usr/lib/python*/ensurepip && \
    pip3 install --upgrade pip setuptools && \
    rm -r /root/.cache

# Install necessary python libs
RUN pip3 install \
    tqdm docker boto3 werkzeug \
    ujson

# Install rust
RUN apk add --update --no-cache \
    libffi-dev \
    rust cargo

# Install battlecode bindings
ADD . /battlecode

WORKDIR /battlecode/bindings
RUN python3 generate.py

WORKDIR /battlecode/bindings/python
RUN pip3 install -r requirements.txt
RUN rustc --version
RUN cargo build #--release
RUN python3 setup.py install

EXPOSE 80

WORKDIR /battlecode/battlecode-manager

CMD ["sh", "start_docker.sh"]
