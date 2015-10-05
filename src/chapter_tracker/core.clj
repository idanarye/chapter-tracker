(ns chapter-tracker.core (:gen-class)
  (:use chapter-tracker.view)
  (:use chapter-tracker.model)
  (:use chapter-tracker.schema)
  (:use chapter-tracker.file-matching)
  (:use chapter-tracker.view.series-and-episode-panel)
  (:import java.io.File)
)

(defn print-clj-trace [exception]
  (println (.toString exception))
  (doseq [trace-element (.getStackTrace exception) :when (.. trace-element getClassName (startsWith "chapter"))]
    (println \tab (.toString trace-element))
  )
)

(defn -main[]
  (try
    (create-tables)
    (catch Exception e))
  (try
    (show-frame)
    (catch Exception e
      (print-clj-trace e)
    )
  )
)

(defn -test[]
  ;(let [directory (chapter-tracker.model/fetch-directory-record 23)]
    ;(println "39 " (:volume directory))
    ;(println "39 " (:series directory))
    ;(println (:recursive directory))
  ;)
  (println (fetch-set-of-serieses-with-unviewed-episodes))
  (seq (for [record (take 2 (modify-series-records-based-on-unviewed-episodes (fetch-series-records)))]
           (println record)))
)
