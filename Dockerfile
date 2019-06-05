FROM fastly/lucet:latest

RUN curl -sL https://deb.nodesource.com/setup_12.x | bash -

RUN apt-get update \
	&& apt-get install -y --no-install-recommends \
	nodejs \
	&& rm -rf /var/lib/apt/lists/*

RUN mkdir /workshop
WORKDIR /workshop

ENV PATH=/opt/lucet/bin:$PATH

EXPOSE 8080
