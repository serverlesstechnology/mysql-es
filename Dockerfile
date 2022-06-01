FROM rust:latest as builder

WORKDIR /home/build
RUN git clone https://github.com/serverlesstechnology/mysql-es.git
WORKDIR /home/build/mysql-es

