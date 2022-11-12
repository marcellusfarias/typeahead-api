# Typeahead API

Typeahead or autocomplete is a common feature that people come across on websites. For example, when you are searching on Google, you will notice a word populates before you finish typing.

That's a very simple version of a typeahead REST API just to practice my Rust skills, specially SMART POINTERS using a B-Tree. It does not has a docker volume for practical reasons and has only two endpoints. The parameters need to be configured on the Dockerfile.

## Specific Requirements

### 1. Initial data

The application considers the file `names.json` in this repository as its initial data. The format of this file is `{ [name1]: [popularityOfName1], [name2]: [popularityOfName2], ... }` **without any particular order**. 

Names don't have strict rules for characters and casing. While most common names start with an uppercase letter and then contain just lowercase letters, you can also find names with more uppercase letters, hyphens, and/or spaces in the middle of it, for example. There is not the case where the same name appears more than once with different casing, btw.

### 2. Environment

The application must consider the following environment variables when starting up:

- `PORT`: the port the application must listen on.
- `SUGGESTION_NUMBER`: the max amount of results the application should return.
- `HOST`: the host to where the application will be deployed to. 

### 3. Persistency

There is no persistency. The data is loaded in the memory and, if restarted, will load the same initial values.

### 4. Endpoints

#### `GET /typeahead/{prefix}`

It optionally receives a prefix in the path and returns an array of objects each one having the `name` and `times` (popularity) properties. The result contains all the names that start with the given `prefix` up to a maximum of `SUGGESTION_NUMBER` names, sorted by highest popularity (`times`) and name in ascending order if they have equal popularity, always leaving the exact match (a name that is exactly the received `prefix`) at the beginning if there is one.

If the `prefix` segment of the path is not given or it's empty (`/typeahead` or `/typeahead/`), it returns the `SUGGESTION_NUMBER` names with the highest popularity and name ascending in case of equal popularity.

It considers the `prefix` in a case insensitive way (so you get the same results for `JA`, `Ja`, `jA` or `ja`) but the responses always return the names in the original casing (as they appear in the initial data).

##### Examples

```bash
$ curl -X GET http://{HOST}:{PORT}/typeahead/ja

[{"name":"Janetta","times":973},{"name":"Janel","times":955},{"name":"Jazmin","times":951},{"name":"Janette","times":947},{"name":"Janet","times":936},{"name":"Janeva","times":929},{"name":"Janella","times":916},{"name":"Janeczka","times":915},{"name":"Jaquelin","times":889},{"name":"Janaya","times":878}]
```

```bash
$ curl -X GET http://{HOST}:{PORT}/typeahead/jan

[{"name":"Jan","times":296},{"name":"Janetta","times":973},{"name":"Janel","times":955},{"name":"Janette","times":947},{"name":"Janet","times":936},{"name":"Janeva","times":929},{"name":"Janella","times":916},{"name":"Janeczka","times":915},{"name":"Janaya","times":878},{"name":"Janine","times":858}]
```

#### `POST /typeahead`

It receives a JSON object with a name as the request body (example: `{ "name": "Joanna" }`), increases the popularity for that name in 1, and returns a `201` status code with an object with `name` and `times` properties considering the new state.

If the given name does not exist in the initial data (`names.json`) then this endpoint should return a 400 HTTP error (no new names will be added, it will only increase the popularity of existing names).

This endpoint must be case insensitive, so request for `{ "name": "JOANNA" }`, `{ "name": "Joanna" }` and `{ "name": "JoAnNa" }` should all work to increase the popularity value for *Joanna*, but the returned name in this request should always be in the original casing.

##### Example

```bash
$ curl -X POST -H "Content-Type: application/json" -d '{"name": "Joanna"}' http://{HOST}:{PORT}/typeahead

{"name":"Joanna","times":441}
```

### 5. Performance

There is a JMeter file for testing performances. I have not gathered reports for it yet.

### 6. How to run

You can run it building the image like `docker build -t typeahead-api .` and running it using `docker run -d -e PORT=65432 -e HOST=localhost -e SUGGESTION_NUMBER=10 typeahead-api`