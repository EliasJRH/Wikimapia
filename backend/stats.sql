-- Not a good idea to run this as a script, more as a holding place for queries 

-- Sort pages by incoming links (Top 100 pages with most incoming links)
SELECT link_title, COUNT(*) as count FROM links GROUP BY link_title ORDER BY count DESC limit 100;

