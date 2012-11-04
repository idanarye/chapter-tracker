(ns chapter-tracker.db
  (:require [clojure.java.jdbc :as sql])
)

(def db {
         :classname "org.sqlite.JDBC"
         :subprotocol "sqlite"
         :subname "chapter_tracker.db3"
        })

(defmacro wrap-connection [& body]
  ;shamelessly stolen from http://boss-level.com/?tag=clojure
  `(if (sql/find-connection)
     (do ~@body)
     (sql/with-connection db ~@body)
   )
)
