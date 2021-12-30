FROM rust:1.40 as builder

WORKDIR /home/build
RUN git clone https://github.com/serverlesstechnology/mysql-es.git
WORKDIR /home/build/mysql-es

