insert into PAGES(page_title) values("Amoeba");
insert into LINKS(page_id, link_title) values(1, "Cell");
insert into LINKS(page_id, link_title) values(1, "Unicellular organism");
insert into LINKS(page_id, link_title) values(1, "Pseudopod");

-- select links.link_title from links join pages on links.page_id = pages.id where page_title="<title>"