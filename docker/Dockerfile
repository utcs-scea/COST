FROM ubuntu:22.04

RUN apt update -y && apt install -y \
    build-essential \
&&  apt remove && apt clean -y

ENV LANG en_US.UTF-8
ENV LANGUAGE en_US:en
ENV LC_ALL en_US.UTF-8


RUN apt update -y && apt install -y \
    locales \
&&  apt remove -y && apt clean -y \
&&  sed -i '/en_US.UTF-8/s/^# //g' /etc/locale.gen && \
    locale-gen en_US.UTF-8

ARG UNAME
ARG UID
ARG GID

RUN if [ "${UNAME}" != "root" ] ; then groupadd -g ${GID} ${UNAME} \
      &&  useradd -ms /bin/bash  -u "${UID}" -g "${GID}" ${UNAME} ; fi

RUN mkdir -p /home/${UNAME} \
      && chown ${UNAME}:${UNAME} /home/${UNAME}

WORKDIR /home/${UNAME}

RUN apt update -y && apt install -y \
    curl \
&&  su ${UNAME} -c "curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y" \
&&  apt remove -y curl && apt clean -y



USER ${UNAME}

RUN . $HOME/.cargo/env \
&&  cargo install cargo-make

USER root

ARG IS_CI=false

RUN if [ "${IS_CI}" = "false" ]; then \
    apt update -y && apt install -y \
    gdb \
    vim \
    git \
    universal-ctags \
    fonts-powerline \
&&  apt remove -y && apt clean -y; fi

USER ${UNAME}

WORKDIR /cost

RUN /bin/bash
