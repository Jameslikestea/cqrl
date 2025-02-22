command "new_post" {
    modelled_by = model.post_input
}

command "update_post" {
    modelled_by = model.post_update_input
}

query "posts" {
    modelled_by = model.post
}

model "post_input" {
    title = {
        type = "string"
        required = true
    }
    body = {
        type = "string"
        required = true
    }
}

model "post_update_input" {
    id = {
        type = "id"
        required = true
    }
    title = {
        type = "string"
        required = false
    }
    body = {
        type = "string"
        required = false
    }
}

model "post" {
    id = {
        type = "id"
        required = true
        primary = true
    }
    title = {
        type = "string"
        required = false
    }
    body = {
        type = "string"
        required = false
    }
    created_at = {
        type = "datetime"
        required = false
    }
} 