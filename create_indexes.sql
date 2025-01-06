create unique index idx_page_titles on PAGES(page_title);
create index idx_links_page_id on LINKS(page_id);
create index idx_redirects_og_page_titles on REDIRECTS(page_title);
