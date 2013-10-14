(ns chapter-tracker.file-matching (:gen-class)
  (:require clojure.string)
  (:use chapter-tracker.db)
  (:use chapter-tracker.model)
  (:require [clojure.java.jdbc :as sql])
  (:import java.io.File)
)

(defn guess-episode[pattern file-name]
  (let [regex (re-pattern pattern)
        matcher (re-matcher regex file-name)
        series-name-in-file-name (re-find regex file-name)]
    (if (.find matcher)
      (try
        [
         (try (.group matcher "v") (catch Exception e nil)) ; Ignore if no volume found
         (.group matcher "c") ; Bail if no episode found
        ]
        (catch Exception e ; In case there is no such group
          (let [rest-of-file-name (.substring file-name (.end matcher))
                assumed-episode-number (re-find #"\d+" rest-of-file-name)]
            [nil (if assumed-episode-number (Integer. assumed-episode-number))]
          )))
      :not-belonging
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
                                                            (let [guess (guess-episode pattern (.getName file))]
                                                              (when-not (= guess :not-belonging)
                                                                (let [[guessed-volume guessed-episode] guess]
                                                                  {:series (:series-id series)
                                                                   :volume guessed-volume
                                                                   :number guessed-episode
                                                                   :file (.toString file)
                                                                   :name (str (:series-name series) (if guessed-volume (str " v" guessed-volume) "") " c" (or guessed-episode "???"))
                                                                  })
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
