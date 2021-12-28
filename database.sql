drop table if exists tag cascade;
drop table if exists question cascade;
drop table if exists tag_question cascade;

create table tag (
  tag_id serial primary key,
  tag_title varchar(30) not null
);

create table question (
  question_id serial primary key,
  title varchar(200) not null,
  q_description varchar(1000) not null,
  question_link varchar(200) not null,
  votes integer not null,
  views varchar(20) not null
);

create table tag_question (
  tag_id integer references tag (tag_id) on update cascade on delete cascade,
  question_id integer references question (question_id) on update cascade,
  constraint tag_question_pkey primary key (tag_id,question_id)
);

insert into tag (tag_title) values ('python'),('rust');

insert into question (title,q_description,question_link,votes,views) values ('i dont know rust','we should know we other','https://stackoverflow.com/questions/21716853/error-syntax-error-at-or-near-when-creating-a-new-table',900,'1 million');


insert into tag_question (tag_id,question_id) values (2,1);