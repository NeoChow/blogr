--
-- PostgreSQL database dump
--

-- Dumped from database version 10.1
-- Dumped by pg_dump version 10.1

SET statement_timeout = 0;
SET lock_timeout = 0;
SET idle_in_transaction_session_timeout = 0;
SET client_encoding = 'UTF8';
SET standard_conforming_strings = on;
SET check_function_bodies = false;
SET client_min_messages = warning;
SET row_security = off;

--
-- Name: blog; Type: DATABASE; Schema: -; Owner: -
--

CREATE DATABASE blog WITH TEMPLATE = template0 ENCODING = 'UTF8' LC_COLLATE = 'en_US.UTF-8' LC_CTYPE = 'en_US.UTF-8';


\connect blog

SET statement_timeout = 0;
SET lock_timeout = 0;
SET idle_in_transaction_session_timeout = 0;
SET client_encoding = 'UTF8';
SET standard_conforming_strings = on;
SET check_function_bodies = false;
SET client_min_messages = warning;
SET row_security = off;

--
-- Name: blog; Type: COMMENT; Schema: -; Owner: -
--

COMMENT ON DATABASE blog IS 'A Rust powered, feature rich, fast & efficient, blog.';


--
-- Name: plpgsql; Type: EXTENSION; Schema: -; Owner: -
--

CREATE EXTENSION IF NOT EXISTS plpgsql WITH SCHEMA pg_catalog;


--
-- Name: EXTENSION plpgsql; Type: COMMENT; Schema: -; Owner: -
--

COMMENT ON EXTENSION plpgsql IS 'PL/pgSQL procedural language';


--
-- Name: pgcrypto; Type: EXTENSION; Schema: -; Owner: -
--

CREATE EXTENSION IF NOT EXISTS pgcrypto WITH SCHEMA public;


--
-- Name: EXTENSION pgcrypto; Type: COMMENT; Schema: -; Owner: -
--

COMMENT ON EXTENSION pgcrypto IS 'cryptographic functions';


SET search_path = public, pg_catalog;

--
-- Name: array_unique(anyarray); Type: FUNCTION; Schema: public; Owner: -
--

CREATE FUNCTION array_unique(arr anyarray) RETURNS anyarray
    LANGUAGE sql
    AS $_$
    select array( select distinct unnest($1) )
$_$;


--
-- Name: description(integer, text, text); Type: FUNCTION; Schema: public; Owner: -
--

CREATE FUNCTION description(chars integer, body text, short text DEFAULT NULL::text) RETURNS text
    LANGUAGE plpgsql
    AS $$
-- AS 'function body text'
DECLARE
    rst text;
BEGIN

CASE WHEN (short) IS NOT NULL THEN rst:= short;
     ELSE rst:= LEFT(body, chars); END CASE;

-- CASE short WHEN NOT NULL THEN rst := short;
-- ELSE rst := LEFT(body, chars); END CASE;
return rst;
END
$$;


--
-- Name: fulltxt_articles_update(); Type: FUNCTION; Schema: public; Owner: -
--

CREATE FUNCTION fulltxt_articles_update() RETURNS trigger
    LANGUAGE plpgsql
    AS $$begin
  new.fulltxt := setweight(to_tsvector('pg_catalog.english', new.title), 'A') || 
		 setweight(to_tsvector('pg_catalog.english', coalesce(new.description,'')), 'B') || 
		 setweight(to_tsvector('pg_catalog.english', new.body), 'C');
  new.modified := now() AT TIME ZONE 'UTC';
  return new;
end
$$;


--
-- Name: proc_blog_users_insert(); Type: FUNCTION; Schema: public; Owner: -
--

CREATE FUNCTION proc_blog_users_insert() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
begin
    -- Hash the password with a newly generated salt
    -- crypt() will store the hash and salt (and the algorithm and iterations) in the column
    new.hash_salt := crypt(new.hash_salt, gen_salt('bf', 8));
  return new;
end
$$;


--
-- Name: proc_blog_users_update(); Type: FUNCTION; Schema: public; Owner: -
--

CREATE FUNCTION proc_blog_users_update() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
begin
  IF NEW.hash_salt IS NULL OR NEW.hash_salt = '' THEN
    new.hash_salt := old.hash_salt;
--     new.attempts := 99;
  ELSE 
    -- new.hash_salt := crypt(new.hash_salt, gen_salt('bf', 8));
    new.hash_salt := crypt(new.hash_salt, old.hash_salt);
--     new.attempts := 66;
  END IF;
  return new;
end
$$;


SET default_tablespace = '';

SET default_with_oids = false;

--
-- Name: articles; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE articles (
    aid oid NOT NULL,
    title character varying NOT NULL,
    posted timestamp without time zone NOT NULL,
    body text NOT NULL,
    description character varying,
    tag2 character varying,
    tag character varying[],
    fulltxt tsvector,
    author oid,
    markdown text,
    image character varying,
    modified timestamp without time zone
);


--
-- Name: archive_articles; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE archive_articles (
)
INHERITS (articles);


--
-- Name: articles_aid_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE articles_aid_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: articles_aid_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE articles_aid_seq OWNED BY articles.aid;


--
-- Name: stage; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE stage (
    aid oid DEFAULT nextval('articles_aid_seq'::regclass) NOT NULL,
    title character varying NOT NULL,
    posted timestamp without time zone NOT NULL,
    body text NOT NULL,
    description character varying,
    tag2 character varying,
    tag character varying[],
    fulltxt tsvector,
    author oid,
    markdown text,
    image character varying
);


--
-- Name: users; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE users (
    userid oid NOT NULL,
    username character varying(30) NOT NULL,
    display character varying(60) NOT NULL,
    is_admin boolean NOT NULL,
    hash_salt text NOT NULL,
    attempts smallint DEFAULT 0 NOT NULL,
    lockout timestamp without time zone
);




--
-- Name: users_userid_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE users_userid_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: users_userid_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE users_userid_seq OWNED BY users.userid;


--
-- Name: archive_articles aid; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY archive_articles ALTER COLUMN aid SET DEFAULT nextval('articles_aid_seq'::regclass);


--
-- Name: articles aid; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY articles ALTER COLUMN aid SET DEFAULT nextval('articles_aid_seq'::regclass);


--
-- Name: users userid; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY users ALTER COLUMN userid SET DEFAULT nextval('users_userid_seq'::regclass);


--
-- Data for Name: archive_articles; Type: TABLE DATA; Schema: public; Owner: -
--



--
-- Data for Name: articles; Type: TABLE DATA; Schema: public; Owner: -
--


--
-- Data for Name: stage; Type: TABLE DATA; Schema: public; Owner: -
--



--
-- Data for Name: users; Type: TABLE DATA; Schema: public; Owner: -
--

INSERT INTO users (userid, username, display, is_admin, hash_salt, attempts, lockout) VALUES (1, 'admin', 'Administrator', true, '$2a$08$PLVHtEhTeEJyrqkLcAcuI.sS2j5dnkullXj65Bzovxdcr9npNCI9O', 0, NULL);


--
-- Name: articles_aid_seq; Type: SEQUENCE SET; Schema: public; Owner: -
--

SELECT pg_catalog.setval('articles_aid_seq', 18, true);


--
-- Name: users_userid_seq; Type: SEQUENCE SET; Schema: public; Owner: -
--

SELECT pg_catalog.setval('users_userid_seq', 3, true);


--
-- Name: articles articles_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY articles
    ADD CONSTRAINT articles_pkey PRIMARY KEY (aid);


--
-- Name: users constrait_users_username_unique; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY users
    ADD CONSTRAINT constrait_users_username_unique UNIQUE (username);


--
-- Name: stage stage_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY stage
    ADD CONSTRAINT stage_pkey PRIMARY KEY (aid);


--
-- Name: users_old users_pk_userid; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY users_old
    ADD CONSTRAINT users_pk_userid PRIMARY KEY (userid);


--
-- Name: users users_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY users
    ADD CONSTRAINT users_pkey PRIMARY KEY (userid);


--
-- Name: users_old users_unique_email; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY users_old
    ADD CONSTRAINT users_unique_email UNIQUE (email);


--
-- Name: users_old users_unique_username; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY users_old
    ADD CONSTRAINT users_unique_username UNIQUE (username);


--
-- Name: fulltxt_idx; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX fulltxt_idx ON articles USING gin (fulltxt);


--
-- Name: stage_fulltxt_idx; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX stage_fulltxt_idx ON stage USING gin (fulltxt);


--
-- Name: users trigger_blog_users_insert; Type: TRIGGER; Schema: public; Owner: -
--

CREATE TRIGGER trigger_blog_users_insert BEFORE INSERT ON users FOR EACH ROW EXECUTE PROCEDURE proc_blog_users_insert();


--
-- Name: users trigger_blog_users_update2; Type: TRIGGER; Schema: public; Owner: -
--

CREATE TRIGGER trigger_blog_users_update2 BEFORE UPDATE OF hash_salt ON users FOR EACH ROW EXECUTE PROCEDURE proc_blog_users_update();


--
-- Name: articles update_articles; Type: TRIGGER; Schema: public; Owner: -
--

CREATE TRIGGER update_articles BEFORE INSERT OR UPDATE ON articles FOR EACH ROW EXECUTE PROCEDURE fulltxt_articles_update();


--
-- Name: stage update_stage; Type: TRIGGER; Schema: public; Owner: -
--

CREATE TRIGGER update_stage BEFORE INSERT OR UPDATE ON stage FOR EACH ROW EXECUTE PROCEDURE fulltxt_articles_update();


--
-- PostgreSQL database dump complete
--

