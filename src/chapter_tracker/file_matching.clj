(ns chapter-tracker.file-matching
  (:require clojure.string)
  (:use chapter-tracker.db)
  (:use chapter-tracker.model)
  (:require [clojure.java.jdbc :as sql])
  (:import java.io.File)
)

(defn guess-episode[pattern file-name]
  (let [regex (re-pattern pattern)
        series-name-in-file-name (re-find regex file-name)]
    (if series-name-in-file-name
      (do;then
        (let [series-name-index (.indexOf file-name series-name-in-file-name)
              rest-of-file-name (.substring file-name (+ series-name-index (.length series-name-in-file-name)))
              assumed-episode-number (re-find #"\d+" rest-of-file-name)
             ]
          (if assumed-episode-number (Integer. assumed-episode-number))
        )
      )
      (do;else
        :not-belonging
      )
    )
  )
)

(defmulti find-new-files-for-series number?)
(defmethod find-new-files-for-series true [series-id]
  (find-new-files-for-series (fetch-series-record series-id))
)
(defmethod find-new-files-for-series false [series]
  (let [allowed-types (-> series :media :file-types (clojure.string/split #"\s+") set)
        allowed? (fn [file-name] (-> file-name .toString (clojure.string/split #"\.") last allowed-types))
       ]
    (apply concat (for [directory-record (fetch-directory-records-for series)]
                    (let [dir       (File. (:directory directory-record))
                          pattern   (:pattern directory-record)
                         ]
                      (if (.isDirectory dir)
                        (do;then
                          (let [files (->> dir .listFiles (filter allowed?))
                                files-already-loaded (all-files-for (:series-id series))
                               ]
                            (sort-by :number (remove nil? (for [file (filter #(not (files-already-loaded (.toString %))) files)]
                                                            (let [guessed-episode (guess-episode pattern (.getName file))]
                                                              (when-not (= guessed-episode :not-belonging)
                                                                {:series (:series-id series)
                                                                 :number guessed-episode
                                                                 :file (.toString file)
                                                                 :name (str (:series-name series) " " (or guessed-episode "???"))
                                                                }
                                                              )
                                                            )
                                                          )
                                             )
                            )
                          )
                        )
                        (do;else
                          (throw (RuntimeException. (str (.toString dir) " is not a Directory")))
                        )
                      )
                    )
                  )
    )
  )
)

(defn rescan-series [series]
  (count (store-new-episodes (find-new-files-for-series series)))
)

(defn rescan-all[]
  (doseq [series (fetch-series-records)]
    (print "Loading for" (.toString series) "...")
    (println "loaded" (rescan-series (:series-id series)) "new files")
  )
)
