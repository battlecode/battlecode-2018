



COPY ./docker-artifacts/battlebaby.tar /images/battlebaby.tar
COPY ./docker-artifacts/linux-battlecode-musl /battlecode/battlecode
# this is necessary because we don't need to copy this file into the target dir & it breaks shutil.copytree
RUN rm /battlecode/battlecode/c/lib/libbattlecode.a

ADD battlecode-manager /battlecode/battlecode-manager
ADD battlecode-maps /battlecode/battlecode-maps
WORKDIR /battlecode/battlecode-manager

CMD ["sh", "start_docker.sh"]
