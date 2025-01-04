drop table if exists PAGES;
drop table if exists LINKS;

create table PAGES (
  id integer not null primary key,
  page_title text not null unique
);

create unique index idx_page_titles on PAGES(page_title);

create table LINKS (
  id integer not null primary key,
  page_id integer not null,
  link_title text not null,
  foreign key (page_id) references PAGES(id)
);

create index idx_links_page_id on LINKS(page_id);
