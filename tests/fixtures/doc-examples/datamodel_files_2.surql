-- Set up the in-memory backend
DEFINE BUCKET my_bucket BACKEND "file:/users/your_user_name";

-- Convenience functions to save, decode back into
-- SurrealQL type, and delete
DEFINE FUNCTION fn::save_file($file_name: string, $input: any) {
    LET $file = type::file("shopping_carts", $file_name);
    $file.put(encoding::cbor::encode($input));
};

DEFINE FUNCTION fn::get_file($file_name: string) -> object {
    encoding::cbor::decode(type::file("shopping_carts", $file_name).get())
};

DEFINE FUNCTION fn::delete_file($file_name: string) {
    type::file("shopping_carts", $file_name).delete();
};

-- Save current shopping cart
fn::save_file("temp_cart_user_24567", {
    items: ["shirt1"],
    last_updated: time::now()
});

fn::get_file("temp_cart_user_24567");
-- Returns { items: ['shirt1', 'deck_of_cards'], last_updated: d'2025-11-20T01:03:24.141080Z' }

-- User adds item, save over file with newer information
fn::save_file("temp_cart_user_24567", {
    items: ["shirt1", "deck_of_cards"],
    last_updated: time::now()
});

fn::get_file("temp_cart_user_24567");
-- Returns { items: ['shirt1', 'deck_of_cards'], last_updated: d'2025-11-20T01:06:02.752429Z' }

-- Session is over, delete temp file
fn::delete_file("temp_cart_user_24567");
