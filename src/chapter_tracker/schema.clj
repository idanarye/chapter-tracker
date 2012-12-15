(ns chapter-tracker.schema (:gen-class)
  (:use chapter-tracker.db)
  (:use chapter-tracker.table-definitions)
  (:require [clojure.java.jdbc :as sql])
)

(defn create-tables[]
  (wrap-connection
    (doseq [[table-name table-fields] tables]
      (apply sql/create-table table-name [:id :integer "primary key" :autoincrement](map (fn [field]
                                                                                           (into [
                                                                                                  (field :field-name)
                                                                                                  (field :field-type)
                                                                                                 ](field :field-options))
                                                                                         ) table-fields))
    )
    (sql/do-commands "CREATE UNIQUE INDEX serieses_unique ON serieses(media_type,name);")
  )
)
