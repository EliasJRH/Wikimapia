drop table if exists PAGES;
drop table if exists LINKS;
drop table if exists REDIRECTS;

create table PAGES (
  id integer not null primary key,
  page_title text not null unique
);

create table LINKS (
  id integer not null primary key,
  page_id integer not null,
  link_title text not null,
  foreign key (page_id) references PAGES(id)
);

create table REDIRECTS (
  id integer not null primary key,
  page_title text not null,
  redirect_title text not null
);
