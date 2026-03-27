/**[test]

[[test.results]]
value = "NONE"

*/

DEFINE EVENT purchase_made ON TABLE purchase
    WHEN $before == NONE
    THEN {
        LET $customer = (SELECT * FROM customer WHERE id = $after.customer);
        LET $product = (SELECT * FROM product WHERE id = $after.product);

        RELATE $customer->bought->$product CONTENT {
            quantity: $after.quantity,
            total: $after.total,
            status: 'Pending',
        };

        CREATE log SET
            customer_id = $after.customer,
            product_id = $after.product,
            action = 'purchase_created',
            timestamp = time::now();
    };
