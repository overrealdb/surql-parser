SELECT
    product_id,
    region,
    math::sum(amount) AS total_sales
FROM
    sales
GROUP BY
    product_id, region;
