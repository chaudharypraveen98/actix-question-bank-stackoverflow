## Stackoverflow API with actix web
This api serves the purpose of creating, updating, and reading the questions and tags from the database. We will soon integrate the stackoverflow scraper.

#### How to run : 
1. Set up Postgres database by installing postgress docker image and change the docker-compose.yaml with the database version type. I have used the 12.9 apline version. Docker must be present in your system <a href="https://www.digitalocean.com/community/tutorials/how-to-install-postgresql-on-ubuntu-20-04-quickstart">More Detail</a>
2. Run the database initiate file by following command
   ```sudo psql -h 127.0.0.1 -p 5432 -U actix actix < database.sql```
3. Run the database by `sudo docker-compose up -d`. Be sure to stop the docker after use by using `docker ps` to get the container id then `docker stop <container_id>` to stop the database instance.
4. Run the server by following `cargo run`

#### Database Access
use the

#### Api Endpoints :
1. Hello world : GET REQUEST `http://127.0.0.1:8000/`
2. Get Tags :  GET REQUEST `http://127.0.0.1:8000/tags/`
3. Get Questions : GET REQUEST `http://127.0.0.1:8000/questions/`
4. Get Questions By Tag : 
  * Api structure : GET REQUEST `http://127.0.0.1:8000/questions/<tag_id/>`
  * Sample Api Endpoint : GET REQUEST `http://127.0.0.1:8000/questions/2/`
5. Create Tag : POST REQUEST
  * Api endpoint : `http://127.0.0.1:8000/tags/`
  * Sample Body : 
  * ```{    "tag_title":"c++"    }```
  * Note : Content-Type must be **application/json** in request header
6. Update Tag : PUT REQUEST
   * Api endpoint : `http://127.0.0.1:8000/tags/`
   * Sample body
   * ```{    "tag_title":"golang",    "tag_id":3}```


#### Future Scopes 
To integrate the stackoverflow scraper to get the question requested by the user and saving it in the database for future use.